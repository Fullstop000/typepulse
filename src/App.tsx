import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Box, Container, Flex, Spinner, Text } from "@chakra-ui/react";
import LogsPage from "./components/logview/LogsPage";
import PageHeader from "./components/layout/PageHeader";
import SettingsPage from "./components/settings/page/SettingsPage";
import Sidebar from "./components/layout/Sidebar";
import StatsPage from "./components/stats/StatsPage";
import { SettingSection } from "./components/settings/types";
import { GroupedRow, ShortcutStatRow, Snapshot, Totals, TrendGranularity } from "./types";
import { buildTrendSeries, parseRowDate } from "./utils/stats";

type FilterRange = "today" | "yesterday" | "7d";

function App() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [activeTab, setActiveTab] = useState<"stats" | "logs" | "settings">("stats");
  const [activeSettingsSection, setActiveSettingsSection] =
    useState<SettingSection>("capture");
  const [typingLogText, setTypingLogText] = useState("");
  const [appLogText, setAppLogText] = useState("");
  const [filterRange, setFilterRange] = useState<FilterRange>("today");
  const [trendGranularity, setTrendGranularity] = useState<TrendGranularity>("5m");
  const [filteredShortcutStats, setFilteredShortcutStats] = useState<ShortcutStatRow[]>([]);

  useEffect(() => {
    let mounted = true;
    const fetchSnapshot = async () => {
      try {
        const [data, shortcutRows] = await Promise.all([
          invoke<Snapshot>("get_snapshot"),
          invoke<ShortcutStatRow[]>("get_shortcut_stats_by_range", { range: filterRange }),
        ]);
        if (mounted) {
          setSnapshot(data);
          setFilteredShortcutStats(shortcutRows);
        }
      } catch (error) {
        if (mounted) {
          setFilteredShortcutStats([]);
        }
        console.error("failed to refresh snapshot", error);
      }
    };
    fetchSnapshot();
    const id = setInterval(fetchSnapshot, 1000);
    return () => {
      mounted = false;
      clearInterval(id);
    };
  }, [filterRange]);

  useEffect(() => {
    if (activeTab !== "logs") return;
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

  const pageTitle = activeTab === "stats" ? "数据" : activeTab === "logs" ? "日志" : "设置";

  const filteredRows = useMemo(() => {
    const rows = snapshot?.rows ?? [];
    const todayStart = new Date();
    todayStart.setHours(0, 0, 0, 0);
    const tomorrowStart = new Date(todayStart);
    tomorrowStart.setDate(tomorrowStart.getDate() + 1);
    const yesterdayStart = new Date(todayStart);
    yesterdayStart.setDate(yesterdayStart.getDate() - 1);
    const sevenDaysStart = new Date(todayStart);
    sevenDaysStart.setDate(sevenDaysStart.getDate() - 6);

    return rows.filter((row) => {
      const date = parseRowDate(row.date);
      if (filterRange === "today") {
        return date >= todayStart && date < tomorrowStart;
      }
      if (filterRange === "yesterday") {
        return date >= yesterdayStart && date < todayStart;
      }
      return date >= sevenDaysStart && date < tomorrowStart;
    });
  }, [snapshot, filterRange]);

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
    return Array.from(grouped.values()).sort((a, b) => b.active_typing_ms - a.active_typing_ms);
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

  return (
    <Flex minH="100vh" bg="#f8fafc">
      <Sidebar
        activeTab={activeTab}
        activeSettingsSection={activeSettingsSection}
        onChange={setActiveTab}
        onSettingsSectionChange={setActiveSettingsSection}
      />
      <Box flex="1" overflowY="auto" px={{ base: 4, md: 8 }} py={{ base: 6, md: 8 }}>
        <Container maxW="1100px" px="0" display="flex" flexDirection="column" gap="5">
          {snapshot ? <PageHeader title={pageTitle} /> : null}
          {!snapshot ? (
            <Box bg="white" borderRadius="16px" p="6" boxShadow="0 10px 30px rgba(15,23,42,0.08)">
              <Flex align="center" gap="2">
                <Spinner size="sm" />
                <Text>加载中…</Text>
              </Flex>
            </Box>
          ) : activeTab === "stats" ? (
            <StatsPage
              filterRange={filterRange}
              onFilterChange={setFilterRange}
              totals={totals}
              groupedRows={groupedRows}
              trendSeries={trendSeries}
              trendGranularity={trendGranularity}
              onTrendGranularityChange={setTrendGranularity}
              shortcutRows={filteredShortcutStats}
            />
          ) : activeTab === "logs" ? (
            <LogsPage
              typingLogText={typingLogText}
              appLogText={appLogText}
              onRefreshTyping={async () => setTypingLogText(await invoke<string>("get_log_tail"))}
              onRefreshApp={async () => setAppLogText(await invoke<string>("get_app_log_tail"))}
            />
          ) : (
            <SettingsPage section={activeSettingsSection} snapshot={snapshot} onSnapshotChange={setSnapshot} />
          )}
        </Container>
      </Box>
    </Flex>
  );
}

export default App;
