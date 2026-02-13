import {
  Accordion,
  Badge,
  Box,
  HStack,
  Stack,
  Text,
} from "@chakra-ui/react";
import { ShortcutStatRow } from "../../types";
import { glassSubtleStyle, glassSurfaceStyle } from "../../styles/glass";
import {
  formatShortcutLabel,
  shortcutActionLabel,
} from "../../utils/shortcutMap";

type ShortcutUsagePanelProps = {
  rows: ShortcutStatRow[];
};

type BadgeType = "clipboard" | "undo" | "switch";

const BAR_FILL_GRADIENT =
  "linear-gradient(90deg, rgba(147, 197, 253, 0.27) 0%, rgba(191, 219, 254, 0.20) 100%)";

const BADGE_CONFIG: Record<
  BadgeType,
  {
    mark: string;
    title: string;
    desc: string;
    ring: string;
    bg: string;
    border: string;
  }
> = {
  clipboard: {
    mark: "CP",
    title: "剪贴板永动机",
    desc: "复制粘贴停不下来",
    ring: "linear-gradient(135deg, #2563eb 0%, #60a5fa 100%)",
    bg: "blue.50",
    border: "blue.200",
  },
  undo: {
    mark: "Z",
    title: "时光倒流",
    desc: "撤销操作不仅是后悔药",
    ring: "linear-gradient(135deg, #c2410c 0%, #fdba74 100%)",
    bg: "orange.50",
    border: "orange.200",
  },
  switch: {
    mark: "TAB",
    title: "窗口蹦迪王",
    desc: "在应用间反复横跳",
    ring: "linear-gradient(135deg, #7c3aed 0%, #c4b5fd 100%)",
    bg: "purple.50",
    border: "purple.200",
  },
};

// Build lightweight badge labels from shortcut ranking distribution.
function buildEfficiencyBadges(rows: ShortcutStatRow[]): BadgeType[] {
  const byId = new Map<string, number>(
    rows.map((row) => [row.shortcut_id, row.count]),
  );
  const copyPaste =
    (byId.get("cmd_c") ?? 0) +
    (byId.get("cmd_v") ?? 0) +
    (byId.get("ctrl_c") ?? 0) +
    (byId.get("ctrl_v") ?? 0);
  const undoCount = (byId.get("cmd_z") ?? 0) + (byId.get("ctrl_z") ?? 0);
  const appSwitchCount = byId.get("cmd_tab") ?? 0;
  const badges: BadgeType[] = [];
  if (copyPaste >= 30) {
    badges.push("clipboard");
  }
  if (undoCount >= 20) {
    badges.push("undo");
  }
  if (appSwitchCount >= 20) {
    badges.push("switch");
  }
  return badges;
}

function ShortcutUsagePanel({ rows }: ShortcutUsagePanelProps) {
  const topRows = rows.slice(0, 5);
  const badges = buildEfficiencyBadges(rows);
  const maxCount = Math.max(...topRows.map((r) => r.count), 0);

  return (
    <Box {...glassSurfaceStyle} borderRadius="16px" p="6" h="full">
      <HStack justify="space-between" mb="4" align="center">
        <Text fontSize="xl" fontWeight="semibold">
          快捷键使用统计
        </Text>
        <Text fontSize="sm" color="gray.600">
          Top 5
        </Text>
      </HStack>

      {badges.length > 0 && (
        <Box mb="5">
          <Text fontSize="sm" color="gray.700" mb="3" fontWeight="semibold">
            徽章墙
          </Text>
          <HStack wrap="wrap" gap="3" align="stretch">
            {badges.map((key) => {
              const config = BADGE_CONFIG[key];
              return (
                <Box
                  key={key}
                  bg="rgba(255,255,255,0.52)"
                  p="2.5"
                  borderRadius="xl"
                  borderWidth="1px"
                  borderColor="glass.borderSoft"
                  transition="all 0.2s"
                  minW="190px"
                  _hover={{ transform: "translateY(-2px)", boxShadow: "sm", bg: "rgba(255,255,255,0.62)" }}
                >
                  <HStack gap="3">
                    <Box
                      w="34px"
                      h="34px"
                      borderRadius="full"
                      bg={config.ring}
                      color="white"
                      display="flex"
                      alignItems="center"
                      justifyContent="center"
                      fontSize="2xs"
                      fontWeight="bold"
                      letterSpacing="0.2px"
                      boxShadow="inset 0 0 0 2px rgba(255,255,255,0.32)"
                      flexShrink={0}
                    >
                      {config.mark}
                    </Box>
                    <Stack gap="0">
                      <Text fontSize="xs" fontWeight="bold" lineHeight="1.2">
                        {config.title}
                      </Text>
                      <Text fontSize="2xs" color="gray.600" lineHeight="1.2">
                        {config.desc}
                      </Text>
                    </Stack>
                  </HStack>
                </Box>
              );
            })}
          </HStack>
        </Box>
      )}

      {topRows.length === 0 ? (
        <Text color="gray.500" py="4">
          暂无快捷键数据（仅统计包含 Cmd 或 Ctrl 的组合）。
        </Text>
      ) : (
        <Accordion.Root collapsible defaultValue={[]}>
          {topRows.map((row, index) => {
            const percentage = maxCount > 0 ? (row.count / maxCount) * 100 : 0;
            return (
              <Accordion.Item
                key={row.shortcut_id}
                value={row.shortcut_id}
                {...glassSubtleStyle}
                borderRadius="12px"
                mb="3"
                overflow="hidden"
              >
                <Accordion.ItemTrigger px="4" py="3" position="relative">
                  <Box
                    position="absolute"
                    top="0"
                    bottom="0"
                    left="0"
                    width={`${percentage}%`}
                    bg={BAR_FILL_GRADIENT}
                    opacity="0.9"
                    zIndex={0}
                  />
                  <HStack
                    justify="space-between"
                    w="full"
                    position="relative"
                    zIndex={1}
                  >
                    <HStack gap="3">
                      <Badge colorPalette="blue" variant="subtle">
                        #{index + 1}
                      </Badge>
                      <Stack gap="0">
                        <Text fontWeight="semibold">
                          {formatShortcutLabel(row.shortcut_id)}
                        </Text>
                        <Text fontSize="xs" color="gray.600">
                          {shortcutActionLabel(row.shortcut_id)}
                        </Text>
                      </Stack>
                    </HStack>
                    <HStack gap="3">
                      <Text fontWeight="semibold">{row.count} 次</Text>
                      <Accordion.ItemIndicator />
                    </HStack>
                  </HStack>
                </Accordion.ItemTrigger>
                <Accordion.ItemContent>
                  <Accordion.ItemBody pt="2" px="4" pb="4">
                    <Text fontSize="xs" color="gray.600" mb="2">
                      主要使用应用
                    </Text>
                    <Stack gap="2">
                      {row.apps.slice(0, 5).map((app) => (
                        <HStack
                          key={`${row.shortcut_id}-${app.app_name}`}
                          justify="space-between"
                        >
                          <Text fontSize="sm" color="gray.700" lineClamp={1}>
                            {app.app_name}
                          </Text>
                          <Badge variant="outline" colorPalette="gray">
                            {app.count}
                          </Badge>
                        </HStack>
                      ))}
                    </Stack>
                  </Accordion.ItemBody>
                </Accordion.ItemContent>
              </Accordion.Item>
            );
          })}
        </Accordion.Root>
      )}
    </Box>
  );
}

export default ShortcutUsagePanel;
