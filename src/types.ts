export type StatsRow = {
  date: string;
  app_name: string;
  window_title: string;
  active_typing_ms: number;
  key_count: number;
  session_count: number;
};

export type Snapshot = {
  rows: StatsRow[];
  paused: boolean;
  auto_paused: boolean;
  auto_pause_reason: "blacklist" | "secure_input" | null;
  keyboard_active: boolean;
  ignore_key_combos: boolean;
  excluded_bundle_ids: string[];
  one_password_suggestion_pending: boolean;
  tray_display_mode: MenuBarDisplayMode;
  last_error: string | null;
  log_path: string;
  shortcut_stats: ShortcutStatRow[];
};

export type MenuBarDisplayMode = "icon_only" | "text_only" | "icon_text";

export type GroupedRow = {
  app_name: string;
  active_typing_ms: number;
  key_count: number;
  session_count: number;
};

export type Totals = {
  active: number;
  keys: number;
  sessions: number;
};

export type TrendGranularity = "1m" | "5m" | "1h" | "1d";

export type TrendSeries = {
  timestamps: number[];
  activeSeconds: number[];
  keyCounts: number[];
  averageActiveSecondsPerSession: number[];
  averageKeysPerSession: number[];
};

export type RunningAppInfo = {
  bundle_id: string;
  name: string;
};

export type ShortcutAppUsageRow = {
  app_name: string;
  count: number;
};

export type ShortcutStatRow = {
  shortcut_id: string;
  count: number;
  apps: ShortcutAppUsageRow[];
};
