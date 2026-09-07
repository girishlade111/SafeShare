import { EmptyState } from "../components/EmptyState";

export function HistoryScreen() {
  return (
    <div className="space-y-6">
      <Header
        title="History"
        subtitle="A local-only ledger of sanitization and restore runs."
      />
      <EmptyState
        badge="Empty state"
        title="No history yet"
        description="Sanitize or restore something and the run will be recorded here, stored entirely on this device."
        icon={
          <svg viewBox="0 0 24 24" fill="none" className="w-6 h-6" stroke="currentColor" strokeWidth={1.6}>
            <path d="M3 5h18M3 12h18M3 19h18" />
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