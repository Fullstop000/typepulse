import { Accordion, Box, Grid, HStack, Portal, Text, Tooltip } from "@chakra-ui/react";
import { Totals } from "../../types";
import { formatMs } from "../../utils/stats";

type MetricsGridProps = {
  totals: Totals;
};

type MetricProps = {
  label: string;
  help: string;
  value: string | number;
  funFact?: string;
};

// Build playful descriptions from core metrics so users can quickly grasp scale.
function buildFunFacts(totals: Totals) {
  const activeMinutes = Math.floor(totals.active / 60000);
  const activeSeconds = Math.max(0, Math.floor(totals.active / 1000));
  
  // Constants for calculations
  const songDurationMin = 4; // Average song ~4 mins
  const instantNoodleMin = 3; // 3 mins to cook noodles
  const movieFrameRate = 24; // 24 fps
  const daySeconds = 86400;
  
  // Key constants
  const keyStrokeDistanceMm = 4; // ~4mm travel distance per key
  const fingerTravelCm = 2; // ~2cm finger travel per key (average)
  const chineseCharStrokes = 3; // Avg keystrokes per Chinese character
  
  // Heights & Distances
  const eiffelTowerM = 330;
  
  // Literary works (approximate keystrokes/words)
  const oldManAndSeaWords = 27000; // ~27k words
  const gaokaoEssayChars = 800; // 800 chars
  
  // Calculations
  const fingerDistanceKm = (totals.keys * fingerTravelCm) / 100000;
  const keyStackHeightM = (totals.keys * keyStrokeDistanceMm) / 1000;
  const estimatedChineseChars = Math.floor(totals.keys / chineseCharStrokes);
  const estimatedEnglishWords = Math.floor(totals.keys / 6);
  
  const timeFactCandidates = [
    `你的活跃输入时长相当于听了 ${Math.max(1, Math.floor(activeMinutes / songDurationMin))} 首歌。`,
    `这段专注的时间，足够煮 ${Math.max(1, Number((activeMinutes / instantNoodleMin).toFixed(1)))} 碗泡面了。`,
    `如果把你输入的每一秒都变成一帧电影，你可以拍一部 ${Math.max(1, Math.floor(activeSeconds / movieFrameRate))} 秒的微电影。`,
    `你的键盘敲击时长占比达到了 ${(activeSeconds / daySeconds * 100).toFixed(4)}% 的“全天进度条”。`,
    `在这 ${formatMs(totals.active)} 里，地球已经带着你公转了约 ${(activeSeconds * 29.78).toFixed(1)} 公里。`,
    `按平均阅读速度，这段时间足够阅读约 ${Math.floor(activeMinutes * 300)} 个字的文章。`,
  ];

  const keyFactCandidates = [
    `假设手指每次按键移动 2 厘米，你的手指已经在键盘上“跑”了 ${fingerDistanceKm.toFixed(3)} 公里。`,
    `如果每次按键行程 4 毫米，你按下的键程总高度相当于 ${keyStackHeightM.toFixed(1)} 米，约等于 ${(keyStackHeightM / eiffelTowerM).toFixed(2)} 座埃菲尔铁塔。`,
    `按每个汉字平均 3 次按键计算，你可能已经输出了约 ${estimatedChineseChars} 个汉字，相当于 ${(estimatedChineseChars / gaokaoEssayChars).toFixed(1)} 篇高考作文。`,
    `假设平均 6 次按键一个英文单词，你的输入量相当于 ${(estimatedEnglishWords / oldManAndSeaWords).toFixed(2)} 本《老人与海》。`,
    `你敲击键盘的次数（${totals.keys}），已经超过了许多人一天眨眼的次数（约 1.5 万次）。`,
    `如果每次按键能产生 0.005J 的能量，你产生的总能量可以把一部手机抬高 ${(totals.keys * 0.005 / 0.15 / 9.8).toFixed(1)} 米。`,
  ];

  const timeIndex = Math.abs(activeMinutes + totals.sessions) % timeFactCandidates.length;
  const keyIndex = Math.abs(totals.keys + totals.sessions * 3) % keyFactCandidates.length;

  return {
    time: timeFactCandidates[timeIndex],
    keys: keyFactCandidates[keyIndex],
  };
}

// Compute secondary metrics shown in collapsed details panel.
function buildSegmentInsights(totals: Totals) {
  if (totals.sessions <= 0) {
    return {
      averageDurationPerSegment: "-",
      averageKeysPerSegment: "-",
    };
  }
  const averageDurationMs = Math.round(totals.active / totals.sessions);
  const averageKeys = totals.keys / totals.sessions;
  return {
    averageDurationPerSegment: formatMs(averageDurationMs),
    averageKeysPerSegment: averageKeys.toFixed(1),
  };
}

function Metric({ label, help, value, funFact }: MetricProps) {
  return (
    <Box>
      <HStack gap="2" mb="2">
        <Text fontSize="sm" color="gray.600">{label}</Text>
        <Tooltip.Root openDelay={150} closeDelay={50} positioning={{ placement: "top" }}>
          <Tooltip.Trigger asChild>
            <Box
              as="span"
              fontSize="xs"
              borderRadius="full"
              bg="gray.200"
              px="2"
              py="0.5"
              cursor="help"
              aria-label={`${label}说明`}
            >
              ⓘ
            </Box>
          </Tooltip.Trigger>
          <Portal>
            <Tooltip.Positioner>
              <Tooltip.Content maxW="260px" fontSize="xs">
                {help}
                <Tooltip.Arrow>
                  <Tooltip.ArrowTip />
                </Tooltip.Arrow>
              </Tooltip.Content>
            </Tooltip.Positioner>
          </Portal>
        </Tooltip.Root>
      </HStack>
      <Text fontSize="2xl" fontWeight="bold">{value}</Text>
      {funFact ? <Text mt="2" fontSize="xs" color="gray.600">{funFact}</Text> : null}
    </Box>
  );
}

function MetricsGrid({ totals }: MetricsGridProps) {
  const funFacts = buildFunFacts(totals);
  const segmentInsights = buildSegmentInsights(totals);
  return (
    <Box bg="white" borderRadius="16px" p="6" boxShadow="sm">
      <Grid templateColumns={{ base: "1fr", md: "repeat(2, 1fr)" }} gap="6">
        <Metric
          label="打字时长"
          help="按键间隔不超过5秒的时间，加起来就是打字时长"
          value={formatMs(totals.active)}
          funFact={funFacts.time}
        />
        <Metric
          label="按键次数"
          help="你一共按了多少次键"
          value={totals.keys}
          funFact={funFacts.keys}
        />
      </Grid>
      <Accordion.Root mt="5" collapsible defaultValue={[]}>
        <Accordion.Item value="more-metrics" borderWidth="1px" borderColor="gray.200" borderRadius="12px">
          <Accordion.ItemTrigger px="4" py="3">
            <HStack justify="space-between" w="full">
              <Text fontSize="sm" fontWeight="semibold" color="gray.700">更多指标</Text>
              <Accordion.ItemIndicator />
            </HStack>
          </Accordion.ItemTrigger>
          <Accordion.ItemContent>
            <Accordion.ItemBody px="4" pb="4">
              <Grid templateColumns={{ base: "1fr", md: "repeat(3, 1fr)" }} gap="4">
                <Metric
                  label="输入段数"
                  help="两次按键隔了超过5秒，就算开始了一个新的输入段。"
                  value={totals.sessions}
                />
                <Metric
                  label="平均每段打字时长"
                  help="总打字时长除以输入段数。"
                  value={segmentInsights.averageDurationPerSegment}
                />
                <Metric
                  label="平均每段按键次数"
                  help="总按键次数除以输入段数。"
                  value={segmentInsights.averageKeysPerSegment}
                />
              </Grid>
            </Accordion.ItemBody>
          </Accordion.ItemContent>
        </Accordion.Item>
      </Accordion.Root>
    </Box>
  );
}

export default MetricsGrid;
