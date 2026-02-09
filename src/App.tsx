import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import LogsPage from "./components/LogsPage";
import PageHeader from "./components/PageHeader";
import SettingsPage from "./components/SettingsPage";
import Sidebar from "./components/Sidebar";
import StatsPage from "./components/StatsPage";
import { GroupedRow, Snapshot, Totals, TrendGranularity } from "./types";
import { buildTrendSeries, parseRowDate } from "./utils/stats";

function App() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [activeTab, setActiveTab] = useState<"stats" | "logs" | "settings">(
    "stats",
  );
  const [typingLogText, setTypingLogText] = useState("");
  const [appLogText, setAppLogText] = useState("");
  const [filterDays, setFilterDays] = useState<1 | 7>(1);
  const [trendGranularity, setTrendGranularity] =
    useState<TrendGranularity>("1m");

  useEffect(() => {
    let mounted = true;
    const fetchSnapshot = async () => {
      const data = await invoke<Snapshot>("get_snapshot");
      if (mounted) {
        setSnapshot(data);
      }
    };
    fetchSnapshot();
    const id = setInterval(fetchSnapshot, 1000);
    return () => {
      mounted = false;
      clearInterval(id);
    };
  }, []);

  useEffect(() => {
    if (activeTab !== "logs") {
      return;
    }
    let mounted = true;
    const fetchLog = async () => {
      const [typingData, appData] = await Promise.all([
        invoke<string>("get_log_tail"),
        invoke<string>("get_app_log_tail"),
      ]);
      if (mounted) {
        setTypingLogText(typingData);
        setAppLogText(appData);
      }
    };
    fetchLog();
    const id = setInterval(fetchLog, 2000);
    return () => {
      mounted = false;
      clearInterval(id);
    };
  }, [activeTab]);

  const pageTitle =
    activeTab === "stats" ? "数据" : activeTab === "logs" ? "日志" : "设置";

  const filteredRows = useMemo(() => {
    const rows = snapshot?.rows ?? [];
    const cutoff = new Date();
    cutoff.setHours(0, 0, 0, 0);
    cutoff.setDate(cutoff.getDate() - (filterDays - 1));
    return rows.filter((row) => {
      const rowDate = parseRowDate(row.date);
      return rowDate >= cutoff;
    });
  }, [snapshot, filterDays]);

  const groupedRows = useMemo(() => {
    const grouped = new Map<string, GroupedRow>();
    for (const row of filteredRows) {
      const entry = grouped.get(row.app_name) || {
        app_name: row.app_name,
        active_typing_ms: 0,
        key_count: 0,
        session_count: 0,
      };
      entry.active_typing_ms += row.active_typing_ms;
      entry.key_count += row.key_count;
      entry.session_count += row.session_count;
      grouped.set(row.app_name, entry);
    }
    return Array.from(grouped.values()).sort(
      (a, b) => b.active_typing_ms - a.active_typing_ms,
    );
  }, [filteredRows]);

  const totals = useMemo<Totals>(
    () =>
      groupedRows.reduce(
        (acc, row) => {
          acc.active += row.active_typing_ms;
          acc.keys += row.key_count;
          acc.sessions += row.session_count;
          return acc;
        },
        { active: 0, keys: 0, sessions: 0 },
      ),
    [groupedRows],
  );

  const trendSeries = useMemo(
    () => buildTrendSeries(snapshot?.rows ?? [], trendGranularity),
    [snapshot, trendGranularity],
  );

  const handleTogglePause = async () => {
    const data = await invoke<Snapshot>("update_paused", {
      paused: !snapshot?.paused,
    });
    setSnapshot(data);
  };

  return (
    <div className="layout">
      <Sidebar activeTab={activeTab} onChange={setActiveTab} />
      <main className="content">
        <div className="content-inner">
          {snapshot ? <PageHeader title={pageTitle} /> : null}
          {!snapshot ? (
            <section className="card">
              <p>加载中…</p>
            </section>
          ) : activeTab === "stats" ? (
            <StatsPage
              snapshot={snapshot}
              filterDays={filterDays}
              onFilterChange={setFilterDays}
              totals={totals}
              groupedRows={groupedRows}
              trendSeries={trendSeries}
              trendGranularity={trendGranularity}
              onTrendGranularityChange={setTrendGranularity}
            />
          ) : activeTab === "logs" ? (
            <LogsPage
              typingLogText={typingLogText}
              appLogText={appLogText}
              onRefreshTyping={async () => {
                const data = await invoke<string>("get_log_tail");
                setTypingLogText(data);
              }}
              onRefreshApp={async () => {
                const data = await invoke<string>("get_app_log_tail");
                setAppLogText(data);
              }}
            />
          ) : (
            <SettingsPage
              paused={snapshot.paused}
              keyboardActive={snapshot.keyboard_active}
              lastError={snapshot.last_error}
              onTogglePause={handleTogglePause}
            />
          )}
        </div>
      </main>
    </div>
  );
}

export default App;
