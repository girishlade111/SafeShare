import type { ScreenKey } from "../lib/nav";

interface NavItem {
  key: ScreenKey;
  label: string;
  hint: string;
  icon: React.ReactNode;
}

const NAV: NavItem[] = [
  {
    key: "sanitize",
    label: "Sanitize",
    hint: "Redact PII from text & files",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" className="w-5 h-5" stroke="currentColor" strokeWidth={1.6}>
        <path d="M12 2 4 5v6c0 5 3.4 9.4 8 11 4.6-1.6 8-6 8-11V5l-8-3Z" />
        <path d="m9 12 2 2 4-4" />
      </svg>
    ),
  },
  {
    key: "restore",
    label: "Restore",
    hint: "Bring sanitized text back",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" className="w-5 h-5" stroke="currentColor" strokeWidth={1.6}>
        <path d="M3 12a9 9 0 1 0 3-6.7" />
        <path d="M3 4v5h5" />
      </svg>
    ),
  },
  {
    key: "history",
    label: "History",
    hint: "Local log of past runs",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" className="w-5 h-5" stroke="currentColor" strokeWidth={1.6}>
        <path d="M3 5h18M3 12h18M3 19h18" />
      </svg>
    ),
  },
  {
    key: "settings",
    label: "Settings",
    hint: "Preferences & engine info",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" className="w-5 h-5" stroke="currentColor" strokeWidth={1.6}>
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3h0a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8v0a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1Z" />
      </svg>
    ),
  },
];

interface SidebarProps {
  active: ScreenKey;
  onSelect: (key: ScreenKey) => void;
}

export function Sidebar({ active, onSelect }: SidebarProps) {
  return (
    <aside className="w-64 shrink-0 bg-bg-panel border-r border-line flex flex-col">
      <div className="px-5 py-5 border-b border-line">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded-lg bg-accent-soft border border-line flex items-center justify-center">
            <svg viewBox="0 0 24 24" fill="none" className="w-4 h-4 text-accent" stroke="currentColor" strokeWidth={1.8}>
              <path d="M12 2 4 5v6c0 5 3.4 9.4 8 11 4.6-1.6 8-6 8-11V5l-8-3Z" />
              <path d="m9 12 2 2 4-4" />
            </svg>
          </div>
          <div>
            <div className="text-sm font-semibold leading-tight">SafeShare</div>
            <div className="text-[11px] text-ink-dim leading-tight">Offline PII toolkit</div>
          </div>
        </div>
      </div>

      <nav className="flex-1 px-3 py-3 space-y-1">
        {NAV.map((item) => {
          const isActive = active === item.key;
          return (
            <button
              key={item.key}
              onClick={() => onSelect(item.key)}
              className={[
                "w-full text-left px-3 py-2 rounded-lg flex items-start gap-3 transition-colors",
                isActive
                  ? "bg-accent-soft border border-line"
                  : "border border-transparent hover:bg-bg-soft hover:border-line",
              ].join(" ")}
            >
              <span
                className={[
                  "mt-0.5 shrink-0",
                  isActive ? "text-accent" : "text-ink-muted",
                ].join(" ")}
              >
                {item.icon}
              </span>
              <span className="min-w-0">
                <span
                  className={[
                    "block text-sm font-medium",
                    isActive ? "text-ink" : "text-ink",
                  ].join(" ")}
                >
                  {item.label}
                </span>
                <span className="block text-[11px] text-ink-dim leading-snug">
                  {item.hint}
                </span>
              </span>
            </button>
          );
        })}
      </nav>

      <div className="px-5 py-3 border-t border-line text-[11px] text-ink-dim">
        100% offline · no network
      </div>
    </aside>
  );
}