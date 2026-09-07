import { useEffect, useState } from "react";
import { EmptyState } from "../components/EmptyState";
import { getEngineInfo } from "../lib/bridge";

export function SettingsScreen() {
  const [info, setInfo] = useState<{ name: string; version: string } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    getEngineInfo()
      .then((res) => {
        if (!cancelled) setInfo(res);
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="space-y-6">
      <Header
        title="Settings"
        subtitle="App preferences and information about the offline engine."
      />

      <section className="ss-card p-5">
        <h3 className="text-sm font-semibold text-ink">Engine</h3>
        <p className="mt-1 text-xs text-ink-muted">
          Reports the local Rust engine version reported by the <code>pii_engine</code> workspace member.
        </p>

        <div className="mt-4 grid grid-cols-2 gap-4 text-sm">
          <Row label="Name">
            {info ? info.name : error ? "—" : "Loading…"}
          </Row>
          <Row label="Version">
            {info ? info.version : error ? "—" : "Loading…"}
          </Row>
          <Row label="Source">
            <code className="text-xs">src-tauri/pii_engine</code>
          </Row>
          <Row label="Network">
            <span className="text-emerald-400">Disabled (offline)</span>
          </Row>
        </div>

        {error ? (
          <p className="mt-3 text-xs text-rose-400">
            Could not reach the engine: {error}
          </p>
        ) : null}
      </section>

      <EmptyState
        badge="Empty state"
        title="Preferences will live here"
        description="Theme, default redaction style, and key storage options will be added in a later milestone."
        icon={
          <svg viewBox="0 0 24 24" fill="none" className="w-6 h-6" stroke="currentColor" strokeWidth={1.6}>
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3" />
          </svg>
        }
      />
    </div>
  );
}

function Header({ title, subtitle }: { title: string; subtitle: string }) {
  return (
    <div>
      <h1 className="text-2xl font-semibold text-ink">{title}</h1>
      <p className="mt-1 text-sm text-ink-muted">{subtitle}</p>
    </div>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-1">
      <span className="text-[11px] uppercase tracking-wide text-ink-dim">
        {label}
      </span>
      <span className="text-ink">{children}</span>
    </div>
  );
}