import { GroupedRow, Snapshot, Totals, TrendGranularity, TrendSeries } from "../../types";
import AppTable from "./AppTable";
import FilterBar from "./FilterBar";
import MetricsGrid from "./MetricsGrid";
import StatusCard from "./StatusCard";
import TrendChart from "./TrendChart";

type StatsPageProps = {
  snapshot: Snapshot;
  filterRange: "today" | "yesterday" | "7d";
  onFilterChange: (value: "today" | "yesterday" | "7d") => void;
  totals: Totals;
  groupedRows: GroupedRow[];
  trendSeries: TrendSeries;
  trendGranularity: TrendGranularity;
  onTrendGranularityChange: (value: TrendGranularity) => void;
};

function StatsPage({
  snapshot,
  filterRange,
  onFilterChange,
  totals,
  groupedRows,
  trendSeries,
  trendGranularity,
  onTrendGranularityChange,
}: StatsPageProps) {
  return (
    <>
      <StatusCard snapshot={snapshot} />
      <FilterBar filterRange={filterRange} onChange={onFilterChange} />
      <MetricsGrid totals={totals} />
      <TrendChart
        series={trendSeries}
        granularity={trendGranularity}
        onGranularityChange={onTrendGranularityChange}
      />
      <AppTable rows={groupedRows} />
    </>
  );
}

export default StatsPage;
