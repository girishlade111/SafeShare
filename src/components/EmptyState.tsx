import type { ReactNode } from "react";

interface EmptyStateProps {
  title: string;
  description: string;
  icon: ReactNode;
  badge?: string;
}

/**
 * Reusable empty-state card shown by every screen until real content lands.
 * Kept intentionally minimal — just enough chrome to anchor the layout.
 */
export function EmptyState({ title, description, icon, badge }: EmptyStateProps) {
  return (
    <div className="ss-card flex flex-col items-center justify-center text-center px-8 py-16">
      <div className="w-12 h-12 rounded-xl bg-bg-soft border border-line flex items-center justify-center text-accent mb-4">
        {icon}
      </div>
      {badge ? <div className="ss-pill mb-3">{badge}</div> : null}
      <h2 className="text-lg font-semibold text-ink">{title}</h2>
      <p className="mt-1.5 max-w-md text-sm text-ink-muted">{description}</p>
    </div>
  );
}