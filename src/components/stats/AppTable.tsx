import { Box, HStack, Text } from "@chakra-ui/react";
import { GroupedRow } from "../../types";
import { formatMs } from "../../utils/stats";

type AppTableProps = {
  rows: GroupedRow[];
};

function AppTable({ rows }: AppTableProps) {
  return (
    <Box bg="white" borderRadius="16px" p="6" boxShadow="sm">
      <Text fontSize="xl" fontWeight="semibold" mb="1">详细记录</Text>
      <Text fontSize="sm" color="gray.600" mb="4">仅展示最近 1 天内的数据</Text>
      <Box borderWidth="1px" borderColor="gray.200" borderRadius="12px" overflow="hidden">
        <HStack px="4" py="3" bg="gray.50" fontWeight="semibold" fontSize="sm" justify="space-between">
          <Text flex="2">应用</Text>
          <Text flex="1" textAlign="right">打字</Text>
          <Text flex="1" textAlign="right">按键</Text>
          <Text flex="1" textAlign="right">会话</Text>
        </HStack>
        {rows.length === 0 ? (
          <Text px="4" py="6" color="gray.500" textAlign="center">暂无数据</Text>
        ) : (
          rows.map((row) => (
            <HStack
              key={row.app_name}
              px="4"
              py="3"
              borderTopWidth="1px"
              borderColor="gray.100"
              justify="space-between"
              fontSize="sm"
            >
              <Text flex="2" truncate>{row.app_name}</Text>
              <Text flex="1" textAlign="right">{formatMs(row.active_typing_ms)}</Text>
              <Text flex="1" textAlign="right">{row.key_count}</Text>
              <Text flex="1" textAlign="right">{row.session_count}</Text>
            </HStack>
          ))
        )}
      </Box>
    </Box>
  );
}

export default AppTable;
