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
  keyboard_active: boolean;
  ignore_key_combos: boolean;
  last_error: string | null;
  log_path: string;
};

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
};
