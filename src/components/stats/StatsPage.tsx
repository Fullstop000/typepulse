import {
  DailyTopKeysRow,
  GroupedRow,
  ShortcutStatRow,
  Totals,
  TrendGranularity,
  TrendSeries,
} from "../../types";
import AppTable from "./AppTable";
import DailyTopKeysPanel from "./DailyTopKeysPanel";
import FilterBar from "./FilterBar";
import MetricsGrid from "./MetricsGrid";
import ShortcutUsagePanel from "./ShortcutUsagePanel";
import TrendChart from "./TrendChart";

type StatsPageProps = {
  filterRange: "today" | "yesterday" | "7d";
  onFilterChange: (value: "today" | "yesterday" | "7d") => void;
  totals: Totals;
  groupedRows: GroupedRow[];
  trendSeries: TrendSeries;
  trendGranularity: TrendGranularity;
  onTrendGranularityChange: (value: TrendGranularity) => void;
  shortcutRows: ShortcutStatRow[];
  dailyTopKeysRows: DailyTopKeysRow[];
};

function StatsPage({
  filterRange,
  onFilterChange,
  totals,
  groupedRows,
  trendSeries,
  trendGranularity,
  onTrendGranularityChange,
  shortcutRows,
  dailyTopKeysRows,
}: StatsPageProps) {
  return (
    <>
      <FilterBar filterRange={filterRange} onChange={onFilterChange} />
      <MetricsGrid totals={totals} />
      <DailyTopKeysPanel rows={dailyTopKeysRows} />
      <TrendChart
        series={trendSeries}
        granularity={trendGranularity}
        onGranularityChange={onTrendGranularityChange}
      />
      <ShortcutUsagePanel rows={shortcutRows} />
      <AppTable rows={groupedRows} />
    </>
  );
}

export default StatsPage;
