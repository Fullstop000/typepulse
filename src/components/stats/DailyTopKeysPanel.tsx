import { Badge, Box, Grid, HStack, Stack, Text } from "@chakra-ui/react";
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
    left: "Left Arrow",
    right: "Right Arrow",
    up: "Up Arrow",
    down: "Down Arrow",
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
    <Box bg="white" borderRadius="16px" p="6" boxShadow="sm">
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
        <Grid templateColumns={{ base: "1fr", lg: "repeat(2, 1fr)" }} gap="4">
          {rows.map((row) => (
            <Box key={row.date} borderWidth="1px" borderColor="gray.200" borderRadius="12px" p="4">
              <Text fontSize="sm" fontWeight="semibold" color="gray.700" mb="3">
                {row.date}
              </Text>
              <Stack gap="2">
                {row.keys.map((item, index) => (
                  <HStack key={`${row.date}-${item.key}`} justify="space-between">
                    <HStack gap="2">
                      <Badge colorPalette="blue" variant="subtle">
                        #{index + 1}
                      </Badge>
                      <Text fontSize="sm">{keyLabel(item.key)}</Text>
                    </HStack>
                    <Badge variant="outline" colorPalette="gray">
                      {item.count}
                    </Badge>
                  </HStack>
                ))}
              </Stack>
            </Box>
          ))}
        </Grid>
      )}
    </Box>
  );
}

export default DailyTopKeysPanel;
