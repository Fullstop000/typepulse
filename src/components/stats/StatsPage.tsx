import { Box, Grid, GridItem, Heading, HStack } from "@chakra-ui/react";
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
    <Box>
      <HStack justify="space-between" align="center" mb="6">
        <Heading size="2xl" fontWeight="bold" color="gray.800">
          数据概览
        </Heading>
        <FilterBar filterRange={filterRange} onChange={onFilterChange} />
      </HStack>

      <Box mb="6">
        <MetricsGrid totals={totals} />
      </Box>

      <Grid
        templateColumns={{ base: "1fr", xl: "2fr 1fr" }}
        gap="6"
        mb="6"
        alignItems="start"
      >
        <GridItem minW="0">
          <TrendChart
            series={trendSeries}
            granularity={trendGranularity}
            onGranularityChange={onTrendGranularityChange}
          />
        </GridItem>
        <GridItem minW="0">
          <DailyTopKeysPanel rows={dailyTopKeysRows} />
        </GridItem>
      </Grid>

      <Grid
        templateColumns={{ base: "1fr", xl: "1fr 1fr" }}
        gap="6"
        alignItems="start"
      >
        <GridItem minW="0">
          <ShortcutUsagePanel rows={shortcutRows} />
        </GridItem>
        <GridItem minW="0">
          <AppTable rows={groupedRows} />
        </GridItem>
      </Grid>
    </Box>
  );
}

export default StatsPage;
