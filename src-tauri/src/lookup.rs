use std::{
    collections::{HashMap, HashSet},
    future::Future,
    time::Duration,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub const MAX_LOOKUP_ATTEMPTS: u8 = 3;

/// How long a per-player League Client request may take. Match history makes the client ask
/// Riot's servers, which regularly takes longer than Shaco's 500 ms default.
pub const LOOKUP_TIMEOUT: Duration = Duration::from_secs(10);

/// A per-player lookup that is tried up to `MAX_LOOKUP_ATTEMPTS` times. Lookups run in the
/// background, so a slow answer never holds up the lobby or the scoreboard.
pub struct Lookup<T> {
    pub values: HashMap<String, T>,
    attempts: HashMap<String, u8>,
    pending: HashSet<String>,
    sender: UnboundedSender<(String, anyhow::Result<T>)>,
    receiver: UnboundedReceiver<(String, anyhow::Result<T>)>,
}

impl<T> Default for Lookup<T> {
    fn default() -> Self {
        let (sender, receiver) = unbounded_channel();
        Self {
            values: HashMap::new(),
            attempts: HashMap::new(),
            pending: HashSet::new(),
            sender,
            receiver,
        }
    }
}

impl<T: Clone + Send + 'static> Lookup<T> {
    /// Counts an attempt and returns true if this player still needs a lookup.
    pub fn start_attempt(&mut self, key: &str) -> bool {
        if key.is_empty() || self.values.contains_key(key) || self.pending.contains(key) {
            return false;
        }
        let attempts = self.attempts.entry(key.to_string()).or_default();
        if *attempts >= MAX_LOOKUP_ATTEMPTS {
            return false;
        }
        *attempts += 1;
        true
    }

    /// Starts a background lookup for this player if one is still needed.
    pub fn spawn<F, Fut>(&mut self, key: &str, lookup: F)
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = anyhow::Result<T>> + Send + 'static,
    {
        if !self.start_attempt(key) {
            return;
        }
        self.pending.insert(key.to_string());
        let sender = self.sender.clone();
        let key = key.to_string();
        let lookup = lookup();
        tauri::async_runtime::spawn(async move {
            let _ = sender.send((key, lookup.await));
        });
    }

    /// Stores the results of background lookups that have finished.
    pub fn collect(&mut self, what: &str) {
        while let Ok((key, result)) = self.receiver.try_recv() {
            self.pending.remove(&key);
            self.record(what, &key, result);
        }
    }

    pub fn record(&mut self, what: &str, key: &str, result: anyhow::Result<T>) {
        match result {
            Ok(value) => {
                self.values.insert(key.to_string(), value);
            }
            Err(error) => log_warn!(
                "{what} lookup failed (attempt {}/{MAX_LOOKUP_ATTEMPTS}): {error}",
                self.attempts.get(key).copied().unwrap_or(0)
            ),
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        self.values.get(key).cloned()
    }

    /// True once the player has a value or has used every attempt.
    pub fn is_settled(&self, key: &str) -> bool {
        key.is_empty()
            || self.values.contains_key(key)
            || (!self.pending.contains(key)
                && self.attempts.get(key).copied().unwrap_or(0) >= MAX_LOOKUP_ATTEMPTS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Collects results until the lookup for `key` is no longer running.
    fn wait_for<T: Clone + Send + 'static>(lookup: &mut Lookup<T>, key: &str) {
        for _ in 0..200 {
            lookup.collect("Test");
            if !lookup.pending.contains(key) {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("lookup for {key} never finished");
    }

    /// A lookup that fails the test if Lookup decides to start it.
    fn unexpected(
        reason: &'static str,
    ) -> impl FnOnce() -> std::future::Ready<anyhow::Result<u32>> {
        move || panic!("lookup should not start: {reason}")
    }

    #[test]
    fn runs_lookups_in_the_background() {
        let mut lookup = Lookup::<u32>::default();
        lookup.spawn("player", || async { Ok(7) });
        assert!(!lookup.is_settled("player"), "pending until collected");
        lookup.spawn("player", unexpected("already pending"));

        wait_for(&mut lookup, "player");
        assert_eq!(lookup.get("player"), Some(7));
        lookup.spawn("player", unexpected("already found"));
    }

    #[test]
    fn retries_failed_lookups_then_gives_up() {
        let mut lookup = Lookup::<u32>::default();
        for _ in 0..MAX_LOOKUP_ATTEMPTS {
            lookup.spawn("player", || async { anyhow::bail!("timed out") });
            wait_for(&mut lookup, "player");
        }
        assert!(lookup.is_settled("player"));
        assert_eq!(lookup.get("player"), None);
        lookup.spawn("player", unexpected("out of attempts"));
    }
}
