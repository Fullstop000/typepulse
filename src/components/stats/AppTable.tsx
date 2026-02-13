import { Box, HStack, Text } from "@chakra-ui/react";
import { GroupedRow } from "../../types";
import { formatMs } from "../../utils/stats";

type AppTableProps = {
  rows: GroupedRow[];
};

function AppTable({ rows }: AppTableProps) {
  const maxTime = Math.max(...rows.map((r) => r.active_typing_ms), 0);

  return (
    <Box bg="white" borderRadius="16px" p="6" boxShadow="sm" h="full">
      <Text fontSize="xl" fontWeight="semibold" mb="4">
        详细记录
      </Text>
      <Box borderWidth="1px" borderColor="gray.200" borderRadius="12px" overflow="hidden">
        <HStack px="4" py="3" bg="gray.50" fontWeight="semibold" fontSize="sm" justify="space-between">
          <Text flex="2">应用</Text>
          <Text flex="1" textAlign="right">
            打字时长
          </Text>
          <Text flex="1" textAlign="right">
            按键
          </Text>
          <Text flex="1" textAlign="right">
            会话
          </Text>
        </HStack>
        {rows.length === 0 ? (
          <Text px="4" py="6" color="gray.500" textAlign="center">
            暂无数据
          </Text>
        ) : (
          rows.map((row) => {
            const timePercentage = maxTime > 0 ? (row.active_typing_ms / maxTime) * 100 : 0;
            return (
              <HStack
                key={row.app_name}
                px="4"
                py="3"
                borderTopWidth="1px"
                borderColor="gray.100"
                justify="space-between"
                fontSize="sm"
              >
                <Text flex="2" truncate fontWeight="medium" color="gray.700">
                  {row.app_name}
                </Text>
                <Box flex="1" position="relative" display="flex" justifyContent="flex-end" alignItems="center">
                  <Box
                    position="absolute"
                    left="0"
                    top="1"
                    bottom="1"
                    width={`${timePercentage}%`}
                    bg="green.50"
                    borderRadius="sm"
                    zIndex={0}
                  />
                  <Text position="relative" zIndex={1} textAlign="right" w="full">
                    {formatMs(row.active_typing_ms)}
                  </Text>
                </Box>
                <Text flex="1" textAlign="right">
                  {row.key_count}
                </Text>
                <Text flex="1" textAlign="right">
                  {row.session_count}
                </Text>
              </HStack>
            );
          })
        )}
      </Box>
    </Box>
  );
}

export default AppTable;
