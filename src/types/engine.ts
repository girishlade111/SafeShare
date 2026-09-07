// Shapes returned by the Rust `pii_engine` crate
// (kept in sync with src-tauri/pii_engine/src/lib.rs).

export interface EngineInfo {
  name: string;
  version: string;
}

/** Mirrors `pii_engine::PiiType`. Serde renames these to snake_case. */
export type PiiType =
  | "email"
  | "phone"
  | "ipv4"
  | "ipv6"
  | "credit_card"
  | "iban"
  | "name";

/** Mirrors `pii_engine::PiiMatch`. */
export interface PiiMatch {
  type: PiiType;
  matched_text: string;
  /** Character index — safe to use with `String.prototype.slice`. */
  start_index: number;
  end_index: number;
  /** 0.0–1.0. Checksum-validated types score high; `name` is low. */
  confidence: number;
}
