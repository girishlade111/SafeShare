import { EmptyState } from "../components/EmptyState";

export function SanitizeScreen() {
  return (
    <div className="space-y-6">
      <Header
        title="Sanitize"
        subtitle="Detect and redact PII in text or local files. Nothing leaves your machine."
      />
      <EmptyState
        badge="Empty state"
        title="No content loaded yet"
        description="Drop a file or paste text to scan it with the offline PII engine. The sanitizer will appear here."
        icon={
          <svg viewBox="0 0 24 24" fill="none" className="w-6 h-6" stroke="currentColor" strokeWidth={1.6}>
            <path d="M12 2 4 5v6c0 5 3.4 9.4 8 11 4.6-1.6 8-6 8-11V5l-8-3Z" />
            <path d="m9 12 2 2 4-4" />
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