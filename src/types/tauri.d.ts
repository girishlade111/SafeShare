// Type-side bridge to the Tauri backend commands exposed in src-tauri/src/lib.rs

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export {};