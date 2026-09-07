import { EmptyState } from "../components/EmptyState";

export function RestoreScreen() {
  return (
    <div className="space-y-6">
      <Header
        title="Restore"
        subtitle="Bring sanitized text back to its original form using a key saved on this device."
      />
      <EmptyState
        badge="Empty state"
        title="Nothing to restore"
        description="Once you sanitize text with a stored restore key, the matching item will show up here."
        icon={
          <svg viewBox="0 0 24 24" fill="none" className="w-6 h-6" stroke="currentColor" strokeWidth={1.6}>
            <path d="M3 12a9 9 0 1 0 3-6.7" />
            <path d="M3 4v5h5" />
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