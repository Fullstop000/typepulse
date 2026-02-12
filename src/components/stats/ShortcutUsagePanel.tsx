import { Accordion, Badge, Box, HStack, Stack, Text } from "@chakra-ui/react";
import { ShortcutStatRow } from "../../types";
import { formatShortcutLabel, shortcutActionLabel } from "../../utils/shortcutMap";

type ShortcutUsagePanelProps = {
  rows: ShortcutStatRow[];
};

// Build lightweight badge labels from shortcut ranking distribution.
function buildEfficiencyBadges(rows: ShortcutStatRow[]): string[] {
  const byId = new Map<string, number>(rows.map((row) => [row.shortcut_id, row.count]));
  const copyPaste =
    (byId.get("cmd_c") ?? 0) +
    (byId.get("cmd_v") ?? 0) +
    (byId.get("ctrl_c") ?? 0) +
    (byId.get("ctrl_v") ?? 0);
  const undoCount = (byId.get("cmd_z") ?? 0) + (byId.get("ctrl_z") ?? 0);
  const badges: string[] = [];
  if (copyPaste >= 30) {
    badges.push("搬运工 The Transporter");
  }
  if (undoCount >= 20) {
    badges.push("时光倒流 Time Traveler");
  }
  return badges;
}

function ShortcutUsagePanel({ rows }: ShortcutUsagePanelProps) {
  const topRows = rows.slice(0, 5);
  const badges = buildEfficiencyBadges(rows);

  return (
    <Box bg="white" borderRadius="16px" p="6" boxShadow="sm">
      <HStack justify="space-between" mb="4" align="center">
        <Text fontSize="xl" fontWeight="semibold">
          快捷键使用统计
        </Text>
        <Text fontSize="sm" color="gray.600">
          Top 5
        </Text>
      </HStack>

      {topRows.length === 0 ? (
        <Text color="gray.500" py="4">
          暂无快捷键数据（仅统计包含 Cmd 或 Ctrl 的组合）。
        </Text>
      ) : (
        <Accordion.Root collapsible defaultValue={[]}>
          {topRows.map((row, index) => (
            <Accordion.Item
              key={row.shortcut_id}
              value={row.shortcut_id}
              borderWidth="1px"
              borderColor="gray.200"
              borderRadius="12px"
              mb="3"
            >
              <Accordion.ItemTrigger px="4" py="3">
                <HStack justify="space-between" w="full">
                  <HStack gap="3">
                    <Badge colorPalette="blue" variant="subtle">
                      #{index + 1}
                    </Badge>
                    <Stack gap="0">
                      <Text fontWeight="semibold">{formatShortcutLabel(row.shortcut_id)}</Text>
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
                <Accordion.ItemBody pt="0" px="4" pb="4">
                  <Text fontSize="xs" color="gray.600" mb="2">
                    主要使用应用
                  </Text>
                  <Stack gap="2">
                    {row.apps.slice(0, 5).map((app) => (
                      <HStack key={`${row.shortcut_id}-${app.app_name}`} justify="space-between">
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
          ))}
        </Accordion.Root>
      )}

      {badges.length > 0 ? (
        <Box mt="4">
          <Text fontSize="sm" color="gray.700" mb="2" fontWeight="semibold">
            效率标签
          </Text>
          <HStack wrap="wrap" gap="2">
            {badges.map((badge) => (
              <Badge key={badge} colorPalette="purple">
                {badge}
              </Badge>
            ))}
          </HStack>
        </Box>
      ) : null}
    </Box>
  );
}

export default ShortcutUsagePanel;
