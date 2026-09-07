import { invoke } from "@tauri-apps/api/core";
import type { EngineInfo, PiiMatch } from "../types/engine";

/**
 * Thin wrapper around Tauri's `invoke` so the rest of the app stays
 * framework-agnostic and easy to mock during unit tests.
 *
 * Everything goes through the local Rust process — no network calls.
 */

export async function getEngineInfo(): Promise<EngineInfo> {
  return invoke<EngineInfo>("engine_info");
}

/**
 * Scan text for PII. Backed by the `detect_pii` command, which delegates to
 * `pii_engine::detect`. Offsets are character indices, so a match can be
 * sliced straight out of the source string.
 */
export async function detectPii(text: string): Promise<PiiMatch[]> {
  return invoke<PiiMatch[]>("detect_pii", { text });
}