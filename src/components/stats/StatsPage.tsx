import { GroupedRow, Snapshot, Totals, TrendGranularity, TrendSeries } from "../../types";
import AppTable from "./AppTable";
import FilterBar from "./FilterBar";
import MetricsGrid from "./MetricsGrid";
import StatusCard from "./StatusCard";
import TrendChart from "./TrendChart";

type StatsPageProps = {
  snapshot: Snapshot;
  filterDays: 1 | 7;
  onFilterChange: (value: 1 | 7) => void;
  totals: Totals;
  groupedRows: GroupedRow[];
  trendSeries: TrendSeries;
  trendGranularity: TrendGranularity;
  onTrendGranularityChange: (value: TrendGranularity) => void;
};

function StatsPage({
  snapshot,
  filterDays,
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
      <FilterBar filterDays={filterDays} onChange={onFilterChange} />
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
