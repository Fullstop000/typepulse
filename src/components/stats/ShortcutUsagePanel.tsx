import {
  Accordion,
  Badge,
  Box,
  Grid,
  HStack,
  Stack,
  Text,
} from "@chakra-ui/react";
import { ShortcutStatRow } from "../../types";
import {
  formatShortcutLabel,
  shortcutActionLabel,
} from "../../utils/shortcutMap";

type ShortcutUsagePanelProps = {
  rows: ShortcutStatRow[];
};

type BadgeType = "clipboard" | "undo" | "switch";

const BADGE_CONFIG: Record<
  BadgeType,
  { emoji: string; color: string; title: string; desc: string }
> = {
  clipboard: {
    emoji: "📋",
    color: "blue",
    title: "剪贴板永动机",
    desc: "复制粘贴停不下来",
  },
  undo: {
    emoji: "⏪",
    color: "orange",
    title: "时光倒流",
    desc: "撤销操作不仅是后悔药",
  },
  switch: {
    emoji: "🔀",
    color: "purple",
    title: "窗口蹦迪王",
    desc: "在应用间反复横跳",
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
    <Box bg="white" borderRadius="16px" p="6" boxShadow="sm" h="full">
      <HStack justify="space-between" mb="4" align="center">
        <Text fontSize="xl" fontWeight="semibold">
          快捷键使用统计
        </Text>
        <Text fontSize="sm" color="gray.600">
          Top 5
        </Text>
      </HStack>

      {badges.length > 0 && (
        <Grid
          templateColumns="repeat(auto-fill, minmax(100px, 1fr))"
          gap="3"
          mb="5"
        >
          {badges.map((key) => {
            const config = BADGE_CONFIG[key];
            return (
              <Box
                key={key}
                bg={`${config.color}.50`}
                p="3"
                borderRadius="xl"
                borderWidth="1px"
                borderColor={`${config.color}.100`}
                transition="all 0.2s"
                _hover={{ transform: "translateY(-2px)", boxShadow: "sm" }}
              >
                <Text fontSize="2xl" mb="1">
                  {config.emoji}
                </Text>
                <Text
                  fontSize="xs"
                  fontWeight="bold"
                  color={`${config.color}.700`}
                  lineHeight="1.2"
                  mb="0.5"
                >
                  {config.title}
                </Text>
                <Text
                  fontSize="2xs"
                  color={`${config.color}.600`}
                  lineHeight="1.2"
                >
                  {config.desc}
                </Text>
              </Box>
            );
          })}
        </Grid>
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
                borderWidth="1px"
                borderColor="gray.200"
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
                    bg="purple.50"
                    opacity="0.5"
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
