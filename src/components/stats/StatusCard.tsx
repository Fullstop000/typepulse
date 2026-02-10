import { Badge, Box, Flex, Stack, Text } from "@chakra-ui/react";
import { Snapshot } from "../../types";

type StatusCardProps = {
  snapshot: Snapshot;
};

function StatusCard({ snapshot }: StatusCardProps) {
  return (
    <Box bg="white" borderRadius="16px" p="6" boxShadow="sm">
      <Flex gap="6" flexWrap="wrap">
        <Stack gap="1" minW="160px">
          <Text fontSize="sm" color="gray.600">键盘监听</Text>
          <Badge colorPalette={snapshot.keyboard_active ? "green" : "red"} width="fit-content">
            {snapshot.keyboard_active ? "已启用" : "未启用"}
          </Badge>
        </Stack>
        <Stack gap="1" minW="160px">
          <Text fontSize="sm" color="gray.600">采集状态</Text>
          <Badge colorPalette={snapshot.paused ? "red" : "green"} width="fit-content">
            {snapshot.paused ? "已暂停" : "运行中"}
          </Badge>
        </Stack>
      </Flex>
      {snapshot.last_error ? (
        <Text mt="4" color="red.600" fontSize="sm">
          {snapshot.last_error}
        </Text>
      ) : null}
    </Box>
  );
}

export default StatusCard;
