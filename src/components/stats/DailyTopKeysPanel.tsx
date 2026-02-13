import { Badge, Box, HStack, Stack, Text } from "@chakra-ui/react";
import { KeyUsageRow } from "../../types";
import { glassSubtleStyle, glassSurfaceStyle } from "../../styles/glass";

type DailyTopKeysPanelProps = {
  rows: KeyUsageRow[];
};

const BAR_FILL_GRADIENT =
  "linear-gradient(90deg, rgba(147, 197, 253, 0.27) 0%, rgba(191, 219, 254, 0.20) 100%)";

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
  const maxCount = Math.max(...rows.map((item) => item.count), 0);
  const totalCount = rows.reduce((sum, item) => sum + item.count, 0);

  return (
    <Box {...glassSurfaceStyle} borderRadius="16px" p="6" h="full">
      <HStack justify="space-between" mb="4" align="center">
        <Text fontSize="xl" fontWeight="semibold">
          Top5 按键
        </Text>
        <Text fontSize="sm" color="gray.600">
          按筛选时间聚合
        </Text>
      </HStack>

      {rows.length === 0 ? (
        <Text color="gray.500" py="2">
          当前时间范围内暂无可展示的按键数据。
        </Text>
      ) : (
        <Box {...glassSubtleStyle} borderRadius="12px" p="4">
          <HStack justify="space-between" align="center" mb="3">
            <Text fontSize="sm" fontWeight="semibold" color="gray.700">
              当前筛选范围
            </Text>
            <Text fontSize="xs" color="gray.600">
              Top5 合计 {totalCount}
            </Text>
          </HStack>
          <Stack gap="2.5">
            {rows.map((item, index) => {
              const percentage = maxCount > 0 ? (item.count / maxCount) * 100 : 0;
              return (
                <Box
                  key={`${item.key}-${index}`}
                  position="relative"
                  borderRadius="10px"
                  overflow="hidden"
                  minH="46px"
                  bg="rgba(255,255,255,0.24)"
                  borderWidth="1px"
                  borderColor="glass.borderSoft"
                >
                  <Box
                    position="absolute"
                    top="0"
                    bottom="0"
                    left="0"
                    width={`${percentage}%`}
                    bg={BAR_FILL_GRADIENT}
                    opacity="0.94"
                    zIndex={0}
                  />
                  <HStack
                    justify="space-between"
                    position="relative"
                    zIndex={1}
                    px="3"
                    py="2"
                    borderRadius="8px"
                    _hover={{ bg: "rgba(255,255,255,0.22)" }}
                  >
                    <HStack gap="2.5">
                      <Badge
                        colorPalette="blue"
                        variant="subtle"
                        size="sm"
                        w="6"
                        textAlign="center"
                        justifyContent="center"
                      >
                        {index + 1}
                      </Badge>
                      <Text fontSize="md" fontWeight="semibold">
                        {keyLabel(item.key)}
                      </Text>
                    </HStack>
                    <Badge
                      variant="outline"
                      bg="rgba(255,255,255,0.68)"
                      borderColor="glass.borderSoft"
                      color="gray.700"
                      px="2"
                      py="0.5"
                      fontWeight="semibold"
                    >
                      {item.count}
                    </Badge>
                  </HStack>
                </Box>
              );
            })}
          </Stack>
        </Box>
      )}
    </Box>
  );
}

export default DailyTopKeysPanel;
