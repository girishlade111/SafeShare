export type ScreenKey = "sanitize" | "restore" | "history" | "settings";

export const DEFAULT_SCREEN: ScreenKey = "sanitize";

export const SCREEN_ORDER: ScreenKey[] = [
  "sanitize",
  "restore",
  "history",
  "settings",
];