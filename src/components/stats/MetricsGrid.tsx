import {
  Accordion,
  Box,
  Grid,
  HStack,
  Portal,
  Text,
  Tooltip,
} from "@chakra-ui/react";
import {
  Activity,
  Clock3,
  Keyboard,
  Layers3,
  LucideIcon,
} from "lucide-react";
import { Totals } from "../../types";
import { glassSubtleStyle, glassSurfaceStyle } from "../../styles/glass";
import { formatMs } from "../../utils/stats";

type MetricsGridProps = {
  totals: Totals;
};

type MetricProps = {
  label: string;
  help: string;
  value: string | number;
  icon: LucideIcon;
  iconBg: string;
  iconColor: string;
  isPrimary?: boolean;
  funFact?: string;
};

// Build playful descriptions from core metrics so users can quickly grasp scale.
function buildFunFacts(totals: Totals) {
  const activeMinutes = Math.floor(totals.active / 60000);
  const activeSeconds = Math.max(0, Math.floor(totals.active / 1000));

  // Constants for calculations.
  const songDurationMin = 4; // Average song ~4 mins
  const instantNoodleMin = 3; // 3 mins to cook noodles
  const movieFrameRate = 24; // 24 fps
  const daySeconds = 86400;

  // Key constants.
  const keyStrokeDistanceMm = 4; // ~4mm travel distance per key
  const fingerTravelCm = 2; // ~2cm finger travel per key (average)
  const chineseCharStrokes = 3; // Avg keystrokes per Chinese character

  // Heights & distances.
  const eiffelTowerM = 330;

  // Literary works (approximate keystrokes/words).
  const oldManAndSeaWords = 27000; // ~27k words
  const gaokaoEssayChars = 800; // 800 chars

  // Calculations.
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

  // Rotate facts by coarse buckets to avoid changing on every single keystroke.
  const timeBucket = Math.floor(activeMinutes / 5) + Math.floor(totals.sessions / 2);
  const keyBucket =
    Math.floor(totals.keys / 50) + Math.floor(totals.sessions / 2) * 3;
  const timeIndex = Math.abs(timeBucket) % timeFactCandidates.length;
  const keyIndex = Math.abs(keyBucket) % keyFactCandidates.length;

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

function Metric({
  label,
  help,
  value,
  icon: Icon,
  iconBg,
  iconColor,
  isPrimary = false,
  funFact,
}: MetricProps) {
  const iconSize = isPrimary ? "9" : "8";
  const labelFontSize = isPrimary ? "lg" : "md";
  const valueFontSize = isPrimary
    ? { base: "44px", md: "52px" }
    : { base: "38px", md: "44px" };

  return (
    <Box {...glassSubtleStyle} borderRadius="14px" p={isPrimary ? "5" : "4"}>
      <HStack justify="space-between" align="start" mb={isPrimary ? "3" : "2.5"}>
        <HStack gap="2.5">
          <Box
            display="inline-flex"
            alignItems="center"
            justifyContent="center"
            w={iconSize}
            h={iconSize}
            borderRadius="10px"
            bg={iconBg}
            color={iconColor}
            borderWidth="1px"
            borderColor="glass.borderSoft"
          >
            <Icon size={isPrimary ? 18 : 16} />
          </Box>
          <Text fontSize={labelFontSize} color="gray.700" fontWeight="semibold">
            {label}
          </Text>
        </HStack>
        <Tooltip.Root openDelay={150} closeDelay={50} positioning={{ placement: "top" }}>
          <Tooltip.Trigger asChild>
            <Box
              as="span"
              fontSize="xs"
              borderRadius="full"
              bg="rgba(255,255,255,0.72)"
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
      <Text
        fontSize={valueFontSize}
        fontWeight="bold"
        lineHeight="1"
        letterSpacing="-0.02em"
        color="gray.900"
      >
        {value}
      </Text>
      {funFact ? (
        <Box
          mt="4"
          px="3.5"
          py="3"
          borderRadius="10px"
          bg="rgba(255,255,255,0.52)"
          borderWidth="1px"
          borderColor="glass.borderSoft"
          boxShadow="inset 0 1px 0 rgba(255,255,255,0.38)"
        >
          <Text fontSize="sm" color="gray.700" lineHeight="1.65">
            {funFact}
          </Text>
        </Box>
      ) : null}
    </Box>
  );
}

function MetricsGrid({ totals }: MetricsGridProps) {
  const funFacts = buildFunFacts(totals);
  const segmentInsights = buildSegmentInsights(totals);

  return (
    <Box {...glassSurfaceStyle} borderRadius="16px" p="6">
      <Grid templateColumns={{ base: "1fr", md: "repeat(2, 1fr)" }} gap="5">
        <Metric
          label="打字时长"
          help="按键间隔不超过5秒的时间，加起来就是打字时长"
          value={formatMs(totals.active)}
          icon={Clock3}
          iconBg="blue.100"
          iconColor="blue.700"
          isPrimary
          funFact={funFacts.time}
        />
        <Metric
          label="按键次数"
          help="你一共按了多少次键"
          value={totals.keys}
          icon={Keyboard}
          iconBg="teal.100"
          iconColor="teal.700"
          isPrimary
          funFact={funFacts.keys}
        />
      </Grid>
      <Accordion.Root mt="4" collapsible defaultValue={[]}>
        <Accordion.Item
          value="more-metrics"
          {...glassSubtleStyle}
          borderRadius="12px"
        >
          <Accordion.ItemTrigger px="4" py="3.5">
            <HStack justify="space-between" w="full">
              <Text fontSize="sm" fontWeight="semibold" color="gray.700">
                更多指标
              </Text>
              <Accordion.ItemIndicator />
            </HStack>
          </Accordion.ItemTrigger>
          <Accordion.ItemContent>
            <Accordion.ItemBody px="4" pt="2" pb="4">
              <Grid templateColumns={{ base: "1fr", md: "repeat(3, 1fr)" }} gap="3">
                <Metric
                  label="输入段数"
                  help="两次按键隔了超过5秒，就算开始了一个新的输入段。"
                  value={totals.sessions}
                  icon={Layers3}
                  iconBg="purple.100"
                  iconColor="purple.700"
                />
                <Metric
                  label="平均每段打字时长"
                  help="总打字时长除以输入段数。"
                  value={segmentInsights.averageDurationPerSegment}
                  icon={Activity}
                  iconBg="cyan.100"
                  iconColor="cyan.700"
                />
                <Metric
                  label="平均每段按键次数"
                  help="总按键次数除以输入段数。"
                  value={segmentInsights.averageKeysPerSegment}
                  icon={Activity}
                  iconBg="green.100"
                  iconColor="green.700"
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
