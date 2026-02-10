import { Box, Grid, HStack, Text } from "@chakra-ui/react";
import { Totals } from "../types";
import { formatMs } from "../utils/stats";

type MetricsGridProps = {
  totals: Totals;
};

type MetricProps = {
  label: string;
  help: string;
  value: string | number;
};

function Metric({ label, help, value }: MetricProps) {
  return (
    <Box>
      <HStack gap="2" mb="2">
        <Text fontSize="sm" color="gray.600">{label}</Text>
        <Box
          as="span"
          title={help}
          fontSize="xs"
          borderRadius="full"
          bg="gray.200"
          px="2"
          py="0.5"
          cursor="help"
        >
          ⓘ
        </Box>
      </HStack>
      <Text fontSize="2xl" fontWeight="bold">{value}</Text>
    </Box>
  );
}

function MetricsGrid({ totals }: MetricsGridProps) {
  return (
    <Box bg="white" borderRadius="16px" p="6" boxShadow="sm">
      <Grid templateColumns={{ base: "1fr", md: "repeat(3, 1fr)" }} gap="6">
        <Metric label="打字时长" help="按键间隔不超过5秒的时间，加起来就是打字时长" value={formatMs(totals.active)} />
        <Metric label="按键次数" help="你一共按了多少次键" value={totals.keys} />
        <Metric label="会话次数" help="两次按键隔了超过5秒，就算开始了一次新会话" value={totals.sessions} />
      </Grid>
    </Box>
  );
}

export default MetricsGrid;
