import { Badge, Box, HStack, Stack, Text } from "@chakra-ui/react";
import { DailyTopKeysRow } from "../../types";

type DailyTopKeysPanelProps = {
  rows: DailyTopKeysRow[];
};

// Convert stored key ids to readable labels in daily top-key cards.
function keyLabel(key: string): string {
  const baseMap: Record<string, string> = {
    space: "Space",
    enter: "Enter",
    tab: "Tab",
    esc: "Esc",
    backspace: "Backspace",
    delete: "Delete",
    left: "←",
    right: "→",
    up: "↑",
    down: "↓",
    command: "Cmd",
    shift: "Shift",
    control: "Ctrl",
    option: "Opt",
  };
  if (baseMap[key]) {
    return baseMap[key];
  }
  if (key.length === 1 && /[a-z]/.test(key)) {
    return key.toUpperCase();
  }
  return key;
}

function DailyTopKeysPanel({ rows }: DailyTopKeysPanelProps) {
  return (
    <Box bg="white" borderRadius="16px" p="6" boxShadow="sm" h="full">
      <HStack justify="space-between" mb="4" align="center">
        <Text fontSize="xl" fontWeight="semibold">
          每日按键 Top 5
        </Text>
        <Text fontSize="sm" color="gray.600">
          按天统计
        </Text>
      </HStack>

      {rows.length === 0 ? (
        <Text color="gray.500" py="2">
          当前时间范围内暂无可展示的按键数据。
        </Text>
      ) : (
        <Stack gap="4">
          {rows.map((row) => {
             const maxCount = Math.max(...row.keys.map((k) => k.count), 0);
             return (
              <Box key={row.date} borderWidth="1px" borderColor="gray.200" borderRadius="12px" p="4">
                <Text fontSize="sm" fontWeight="semibold" color="gray.700" mb="3">
                  {row.date}
                </Text>
                <Stack gap="2">
                  {row.keys.map((item, index) => {
                    const percentage = maxCount > 0 ? (item.count / maxCount) * 100 : 0;
                    return (
                      <Box key={`${row.date}-${item.key}`} position="relative" borderRadius="md" overflow="hidden">
                        <Box
                          position="absolute"
                          top="0"
                          bottom="0"
                          left="0"
                          width={`${percentage}%`}
                          bg="blue.50"
                          opacity="0.6"
                          zIndex={0}
                        />
                        <HStack justify="space-between" position="relative" zIndex={1} px="2" py="1">
                          <HStack gap="2">
                            <Badge colorPalette="blue" variant="subtle" size="sm" w="5" textAlign="center" justifyContent="center">
                              {index + 1}
                            </Badge>
                            <Text fontSize="sm" fontWeight="medium">{keyLabel(item.key)}</Text>
                          </HStack>
                          <Text fontSize="xs" color="gray.600">
                            {item.count}
                          </Text>
                        </HStack>
                      </Box>
                    );
                  })}
                </Stack>
              </Box>
            );
          })}
        </Stack>
      )}
    </Box>
  );
}

export default DailyTopKeysPanel;
