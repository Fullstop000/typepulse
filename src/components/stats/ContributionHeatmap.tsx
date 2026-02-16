import { useMemo } from "react";
import { Box, HStack, Portal, Stack, Text, Tooltip } from "@chakra-ui/react";
import { StatsRow } from "../../types";
import { glassSurfaceStyle } from "../../styles/glass";
import { buildContributionHeatmapData } from "../../utils/contribution";
import { formatMs } from "../../utils/stats";
import {
  CONTRIBUTION_LEVEL_COLOR,
  CONTRIBUTION_LEVEL_LABEL,
} from "./contributionTheme";

type ContributionHeatmapProps = {
  rows: StatsRow[];
  embedded?: boolean;
};

const CELL_SIZE = 11;
const CELL_GAP = 4;
const LABEL_W = 24;

const weekLabel = ["一", "二", "三", "四", "五", "六", "日"];

// Render GitHub-style contribution heatmap for recent 3 months.
// `embedded` lets parent cards reuse this view without nested outer glass cards.
function ContributionHeatmap({ rows, embedded = false }: ContributionHeatmapProps) {
  // Daily aggregation is a pure derivation from rows; memoize to avoid recomputation on unrelated renders.
  const data = useMemo(
    () => buildContributionHeatmapData(rows, { monthsBack: 3 }),
    [rows],
  );
  const containerProps = embedded
    ? {}
    : {
        ...glassSurfaceStyle,
        borderRadius: "16px",
        p: "5",
        h: "full",
      };

  return (
    <Box {...containerProps}>
      <HStack justify="space-between" align="end" mb="4" flexWrap="wrap" gap="3">
        <Stack gap="0.5">
          <Text fontSize="lg" fontWeight="semibold" color="gray.900">键盘活跃日历</Text>
        </Stack>
        <HStack gap="3" color="gray.600">
          <Text fontSize="xs">活跃 {data.activeDays} 天</Text>
          <Text fontSize="xs">按键 {data.totalKeyCount}</Text>
        </HStack>
      </HStack>

      <Box overflowX="auto" pb="2">
        <Box minW={`${LABEL_W + data.weekCount * (CELL_SIZE + CELL_GAP)}px`}>
          <Box position="relative" ml={`${LABEL_W}px`} h="16px" mb="2">
            {data.monthLabels.map((label) => (
              <Text
                key={`${label.text}-${label.weekIndex}`}
                position="absolute"
                left={`${label.weekIndex * (CELL_SIZE + CELL_GAP)}px`}
                top="0"
                fontSize="10px"
                color="gray.500"
              >
                {label.text}
              </Text>
            ))}
          </Box>

          <HStack align="start" gap="2" position="relative">
            <Stack w={`${LABEL_W}px`} gap={`${CELL_GAP}px`} pt="1px">
              {weekLabel.map((label, index) => (
                <Text
                  key={label}
                  h={`${CELL_SIZE}px`}
                  lineHeight={`${CELL_SIZE}px`}
                  fontSize="10px"
                  color={index >= 5 ? "gray.400" : "gray.500"}
                  textAlign="center"
                >
                  {label}
                </Text>
              ))}
            </Stack>

            <Box
              display="grid"
              gridTemplateColumns={`repeat(${data.weekCount}, ${CELL_SIZE}px)`}
              gridTemplateRows={`repeat(7, ${CELL_SIZE}px)`}
              gap={`${CELL_GAP}px`}
            >
              {data.cells.map((cell) => {
                const tooltipAccent = CONTRIBUTION_LEVEL_COLOR[cell.level];
                return (
                  <Tooltip.Root
                    key={cell.dateKey}
                    openDelay={120}
                    closeDelay={80}
                    positioning={{ placement: "top" }}
                  >
                    <Tooltip.Trigger asChild>
                      <Box
                        gridColumn={cell.weekIndex + 1}
                        gridRow={cell.dayOfWeek + 1}
                        borderRadius="3px"
                        bg={CONTRIBUTION_LEVEL_COLOR[cell.level]}
                        borderWidth={cell.isToday ? "1px" : "0px"}
                        borderColor={cell.isToday ? "rgba(14,116,144,0.85)" : "transparent"}
                        boxShadow={
                          cell.level > 0
                            ? "inset 0 0 0 1px rgba(255,255,255,0.18)"
                            : "inset 0 0 0 1px rgba(255,255,255,0.08)"
                        }
                        cursor="pointer"
                      />
                    </Tooltip.Trigger>
                    <Portal>
                      <Tooltip.Positioner>
                        <Tooltip.Content
                          px="3"
                          py="2.5"
                          borderRadius="10px"
                          borderWidth="1px"
                          borderColor="glass.borderSoft"
                          bg="rgba(245,251,255,0.94)"
                          boxShadow="0 10px 22px rgba(15,23,42,0.16)"
                          minW="180px"
                        >
                          <Stack gap="1.5">
                            <HStack justify="space-between" align="center">
                              <Text fontSize="xs" color="gray.600">{cell.dateKey}</Text>
                              <Box w="8px" h="8px" borderRadius="full" bg={tooltipAccent} />
                            </HStack>
                            <Text fontSize="sm" fontWeight="semibold" color="gray.800">
                              {CONTRIBUTION_LEVEL_LABEL[cell.level]} · {cell.keyCount} 次按键
                            </Text>
                            <HStack justify="space-between" fontSize="xs" color="gray.600">
                              <Text>打字时长 {formatMs(cell.activeTypingMs)}</Text>
                              <Text>{cell.sessionCount} 段</Text>
                            </HStack>
                          </Stack>
                          <Tooltip.Arrow>
                            <Tooltip.ArrowTip />
                          </Tooltip.Arrow>
                        </Tooltip.Content>
                      </Tooltip.Positioner>
                    </Portal>
                  </Tooltip.Root>
                );
              })}
            </Box>
          </HStack>
        </Box>
      </Box>

    </Box>
  );
}

export default ContributionHeatmap;
