import { createContext, type ReactNode, useContext } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MenuBarDisplayMode, RunningAppInfo, Snapshot } from "../../types";

type SettingsContextValue = {
  // Latest settings snapshot from backend, used as the single source of truth in UI.
  snapshot: Snapshot;
  // Toggle manual pause/resume collection and refresh snapshot.
  togglePause: () => Promise<void>;
  // Toggle combo-key filtering (Ctrl/Alt/Fn/Shift/Cmd + key) and refresh snapshot.
  toggleIgnoreKeyCombos: () => Promise<void>;
  // Update tray display mode and refresh snapshot.
  updateTrayDisplayMode: (mode: MenuBarDisplayMode) => Promise<void>;
  // Add an app bundle ID to exclusion list and refresh snapshot.
  addAppExclusion: (bundleId: string) => Promise<void>;
  // Remove an app bundle ID from exclusion list and refresh snapshot.
  removeAppExclusion: (bundleId: string) => Promise<void>;
  // Read currently running applications for manual selection.
  loadRunningApps: () => Promise<RunningAppInfo[]>;
  // Dismiss first-run 1Password suggestion and refresh snapshot.
  dismissOnePasswordSuggestion: () => Promise<void>;
  // Accept 1Password suggestion (add exclusion) and refresh snapshot.
  acceptOnePasswordSuggestion: () => Promise<void>;
};

const SettingsContext = createContext<SettingsContextValue | null>(null);

type SettingsProviderProps = {
  snapshot: Snapshot;
  onSnapshotChange: (snapshot: Snapshot) => void;
  children: ReactNode;
};

export function SettingsProvider({
  snapshot,
  onSnapshotChange,
  children,
}: SettingsProviderProps) {
  // All settings mutations go through backend commands and return the latest snapshot.
  // We always write that snapshot back to keep UI state in sync with Rust-side state.
  const togglePause = async () => {
    const data = await invoke<Snapshot>("update_paused", {
      paused: !snapshot.paused,
    });
    onSnapshotChange(data);
  };

  const toggleIgnoreKeyCombos = async () => {
    const data = await invoke<Snapshot>("update_ignore_key_combos", {
      ignoreKeyCombos: !snapshot.ignore_key_combos,
    });
    onSnapshotChange(data);
  };

  const updateTrayDisplayMode = async (mode: MenuBarDisplayMode) => {
    const data = await invoke<Snapshot>("update_menu_bar_display_mode", {
      mode,
    });
    onSnapshotChange(data);
  };

  const addAppExclusion = async (bundleId: string) => {
    const data = await invoke<Snapshot>("add_app_exclusion", { bundleId });
    onSnapshotChange(data);
  };

  const removeAppExclusion = async (bundleId: string) => {
    const data = await invoke<Snapshot>("remove_app_exclusion", { bundleId });
    onSnapshotChange(data);
  };

  const loadRunningApps = async () => {
    return invoke<RunningAppInfo[]>("get_running_apps");
  };

  const dismissOnePasswordSuggestion = async () => {
    const data = await invoke<Snapshot>("dismiss_one_password_suggestion");
    onSnapshotChange(data);
  };

  const acceptOnePasswordSuggestion = async () => {
    const data = await invoke<Snapshot>("accept_one_password_suggestion");
    onSnapshotChange(data);
  };

  return (
    <SettingsContext.Provider
      value={{
        snapshot,
        togglePause,
        toggleIgnoreKeyCombos,
        updateTrayDisplayMode,
        addAppExclusion,
        removeAppExclusion,
        loadRunningApps,
        dismissOnePasswordSuggestion,
        acceptOnePasswordSuggestion,
      }}
    >
      {children}
    </SettingsContext.Provider>
  );
}

export function useSettingsContext() {
  const context = useContext(SettingsContext);
  // Fast-fail to make provider wiring mistakes obvious during development.
  if (!context) {
    throw new Error("useSettingsContext must be used within SettingsProvider");
  }
  return context;
}
