#[derive(Default)]
pub struct QueuePauseState {
    last_phase: Option<String>,
    paused: bool,
    cancel_requested: bool,
    awaiting_dodge_resolution: bool,
    phase_generation: u64,
}

pub struct PhaseUpdate {
    pub pause_changed: Option<bool>,
    pub cancel_search: bool,
    pub ready_check_generation: Option<u64>,
    /// Champ Select just ended without a game, whether or not the queue gets paused for it.
    pub dodged: bool,
}

impl QueuePauseState {
    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn reset(&mut self) -> Option<bool> {
        let was_paused = self.paused;
        let next_generation = self.phase_generation.wrapping_add(1);
        *self = Self {
            phase_generation: next_generation,
            ..Self::default()
        };
        was_paused.then_some(false)
    }

    pub fn disable(&mut self) -> Option<bool> {
        let was_paused = self.paused;
        self.paused = false;
        self.cancel_requested = false;
        self.awaiting_dodge_resolution = false;
        self.phase_generation = self.phase_generation.wrapping_add(1);
        was_paused.then_some(false)
    }

    pub fn ready_check_is_current(&self, generation: u64) -> bool {
        self.last_phase.as_deref() == Some("ReadyCheck")
            && self.phase_generation == generation
            && !self.paused
    }

    pub fn should_cancel_search(&self) -> bool {
        self.paused && self.last_phase.as_deref() == Some("Matchmaking")
    }

    pub fn observe_phase(&mut self, phase: &str, enabled: bool) -> PhaseUpdate {
        let previous = self.last_phase.as_deref();
        let entered_ready_check = phase == "ReadyCheck" && previous != Some("ReadyCheck");
        // A game that goes ahead moves on to GameStart; anything else means someone left.
        let dodged = previous == Some("ChampSelect")
            && matches!(phase, "None" | "Lobby" | "Matchmaking" | "ReadyCheck");
        let was_paused = self.paused;
        let mut cancel_search = false;

        if previous != Some(phase) {
            self.phase_generation = self.phase_generation.wrapping_add(1);
        }

        if !enabled
            || matches!(
                phase,
                "ChampSelect" | "GameStart" | "InProgress" | "WaitingForStats" | "EndOfGame"
            )
        {
            self.paused = false;
            self.cancel_requested = false;
            self.awaiting_dodge_resolution = false;
        } else if previous == Some("ChampSelect") && phase == "None" {
            self.awaiting_dodge_resolution = true;
        } else if (previous == Some("ChampSelect") || self.awaiting_dodge_resolution)
            && matches!(phase, "Lobby" | "Matchmaking" | "ReadyCheck")
        {
            // A completed champ select enters GameStart. Returning to the queue or
            // lobby instead means that champ select was abandoned by a player.
            self.paused = true;
            self.cancel_requested = false;
            self.awaiting_dodge_resolution = false;
        } else if self.paused && previous == Some("Lobby") && phase == "Matchmaking" {
            // The player started a fresh search from the lobby.
            self.paused = false;
            self.cancel_requested = false;
        }

        if self.paused && phase == "Matchmaking" && !self.cancel_requested {
            self.cancel_requested = true;
            cancel_search = true;
        }
        if phase == "Lobby" {
            self.cancel_requested = false;
        }

        self.last_phase = Some(phase.to_string());
        PhaseUpdate {
            pause_changed: (self.paused != was_paused).then_some(self.paused),
            cancel_search,
            ready_check_generation: entered_ready_check.then_some(self.phase_generation),
            dodged,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancels_automatic_requeue_after_abandoned_champ_select() {
        let mut state = QueuePauseState::default();
        state.observe_phase("Matchmaking", true);
        state.observe_phase("ReadyCheck", true);
        state.observe_phase("ChampSelect", true);

        let update = state.observe_phase("Matchmaking", true);
        assert_eq!(update.pause_changed, Some(true));
        assert!(update.cancel_search);
        assert!(!state.observe_phase("Matchmaking", true).cancel_search);

        state.observe_phase("Lobby", true);
        let update = state.observe_phase("Matchmaking", true);
        assert_eq!(update.pause_changed, Some(false));
        assert!(!update.cancel_search);
    }

    #[test]
    fn pauses_in_lobby_until_manual_requeue() {
        let mut state = QueuePauseState::default();
        state.observe_phase("ChampSelect", true);
        let update = state.observe_phase("Lobby", true);
        assert_eq!(update.pause_changed, Some(true));
        assert!(!update.cancel_search);
        assert!(state.paused());

        let update = state.observe_phase("Matchmaking", true);
        assert_eq!(update.pause_changed, Some(false));
    }

    #[test]
    fn normal_game_and_disabled_setting_do_not_pause() {
        let mut state = QueuePauseState::default();
        state.observe_phase("ChampSelect", true);
        assert!(!state.observe_phase("GameStart", true).cancel_search);
        assert!(!state.observe_phase("InProgress", true).cancel_search);
        assert!(!state.paused());

        state.observe_phase("ChampSelect", false);
        assert!(!state.observe_phase("Matchmaking", false).cancel_search);
        assert!(!state.paused());
    }

    #[test]
    fn keeps_auto_accept_suppressed_if_ready_check_wins_the_race() {
        let mut state = QueuePauseState::default();
        state.observe_phase("ChampSelect", true);
        let update = state.observe_phase("ReadyCheck", true);
        assert_eq!(update.pause_changed, Some(true));
        assert!(update.ready_check_generation.is_some());
        assert!(state.paused());

        let update = state.observe_phase("Matchmaking", true);
        assert!(update.cancel_search);
    }

    #[test]
    fn reports_each_dodge_once_even_with_the_pause_off() {
        let mut state = QueuePauseState::default();
        state.observe_phase("ChampSelect", false);
        assert!(state.observe_phase("Matchmaking", false).dodged);
        assert!(!state.observe_phase("ReadyCheck", false).dodged);

        state.observe_phase("ChampSelect", false);
        assert!(state.observe_phase("None", false).dodged);
        assert!(!state.observe_phase("Lobby", false).dodged, "the same dodge");

        state.observe_phase("ChampSelect", false);
        assert!(!state.observe_phase("GameStart", false).dodged);
    }

    #[test]
    fn detects_dodge_across_intermediate_none_phase() {
        let mut state = QueuePauseState::default();
        state.observe_phase("ChampSelect", true);
        state.observe_phase("None", true);
        assert!(!state.paused());
        let update = state.observe_phase("Matchmaking", true);
        assert_eq!(update.pause_changed, Some(true));
        assert!(update.cancel_search);
    }

    #[test]
    fn invalidates_old_ready_check_after_requeue_or_reset() {
        let mut state = QueuePauseState::default();
        let first = state
            .observe_phase("ReadyCheck", true)
            .ready_check_generation
            .unwrap();
        assert!(state.ready_check_is_current(first));
        state.observe_phase("Matchmaking", true);
        let second = state
            .observe_phase("ReadyCheck", true)
            .ready_check_generation
            .unwrap();
        assert!(!state.ready_check_is_current(first));
        assert!(state.ready_check_is_current(second));
        state.reset();
        assert!(!state.ready_check_is_current(second));
    }
}
