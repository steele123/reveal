/** A time an automatic setting can wait; `short` fits in a pill, `label` is for menus. */
export interface TimeOption {
  value: number;
  label: string;
  short: string;
}

/** How long auto accept waits after the ready check pops, in milliseconds. */
export const ACCEPT_TIMES: TimeOption[] = [
  { value: 0, label: "Right away", short: "0s" },
  { value: 2000, label: "After 2 sec", short: "2s" },
  { value: 4000, label: "After 4 sec", short: "4s" },
  { value: 6000, label: "After 6 sec", short: "6s" },
  { value: 8000, label: "After 8 sec", short: "8s" },
  { value: 9000, label: "Last second", short: "Last" },
];

/** How long to wait before queueing again, after a game or a long queue, in seconds. */
export const REQUEUE_TIMES: TimeOption[] = [
  { value: 0, label: "Right away", short: "0s" },
  { value: 10, label: "After 10 sec", short: "10s" },
  { value: 20, label: "After 20 sec", short: "20s" },
  { value: 30, label: "After 30 sec", short: "30s" },
  { value: 60, label: "After 60 sec", short: "60s" },
];

/** How long a pause after a dodge lasts, in seconds; 0 waits for you. */
export const DODGE_TIMES: TimeOption[] = [
  { value: 0, label: "Until I queue", short: "∞" },
  { value: 10, label: "10 sec", short: "10s" },
  { value: 30, label: "30 sec", short: "30s" },
  { value: 60, label: "60 sec", short: "60s" },
  { value: 120, label: "2 min", short: "2m" },
];

/** The longest auto-open waits for all five names, in seconds. */
export const OPEN_TIMES: TimeOption[] = [
  { value: 0, label: "Now", short: "0s" },
  { value: 4, label: "Up to 4 sec", short: "4s" },
  { value: 6, label: "Up to 6 sec", short: "6s" },
  { value: 10, label: "Up to 10 sec", short: "10s" },
];

/** When to ban or lock in, as seconds left on League's timer. */
export const SECONDS_LEFT_TIMES: TimeOption[] = [
  { value: 1, label: "Last second", short: "Last" },
  { value: 3, label: "3 sec left", short: "3s" },
  { value: 5, label: "5 sec left", short: "5s" },
  { value: 10, label: "10 sec left", short: "10s" },
];

/** The pill text for `value`, or the first option's when it isn't one of them. */
export function shortTime(options: TimeOption[], value: number): string {
  return (options.find((option) => option.value === value) ?? options[0]).short;
}
