import { Box, Grid, GridItem, Heading, HStack, Stack, Text } from "@chakra-ui/react";
import {
  GroupedRow,
  KeyUsageRow,
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
  topKeysRows: KeyUsageRow[];
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
  topKeysRows,
}: StatsPageProps) {
  return (
    <Box>
      <HStack justify="space-between" align="center" mb="6">
        <Stack gap="1">
          <Heading size="2xl" fontWeight="bold" color="gray.800">
            数据概览
          </Heading>
          <Text fontSize="sm" color="gray.600">
            你的输入节奏、应用分布和效率偏好，都在这里一屏看完。
          </Text>
        </Stack>
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
          <DailyTopKeysPanel rows={topKeysRows} />
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
