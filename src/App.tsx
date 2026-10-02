import { useState } from "react";
import { Sidebar } from "./components/Sidebar";
import { DEFAULT_SCREEN, SCREEN_ORDER, type ScreenKey } from "./lib/nav";
import { SanitizeScreen } from "./screens/SanitizeScreen";
import { RestoreScreen } from "./screens/RestoreScreen";
import { HistoryScreen } from "./screens/HistoryScreen";
import { SettingsScreen } from "./screens/SettingsScreen";

function App() {
  const [screen, setScreen] = useState<ScreenKey>(DEFAULT_SCREEN);

  return (
    <div className="h-full w-full flex bg-bg text-ink">
      <Sidebar
        active={screen}
        onSelect={(k) => {
          if (SCREEN_ORDER.includes(k)) setScreen(k);
        }}
      />
      <main className="flex-1 min-w-0 overflow-auto">
        <div className="max-w-4xl mx-auto px-8 py-8">{renderScreen(screen)}</div>
      </main>
    </div>
  );
}

function renderScreen(key: ScreenKey) {
  switch (key) {
    case "sanitize":
      return <SanitizeScreen />;
    case "restore":
      return <RestoreScreen />;
    case "history":
      return <HistoryScreen />;
    case "settings":
      return <SettingsScreen />;
  }
}
export default App;
