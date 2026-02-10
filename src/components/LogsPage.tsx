import { Box, Button, Grid, HStack, Text } from "@chakra-ui/react";

type LogsPageProps = {
  typingLogText: string;
  appLogText: string;
  onRefreshTyping: () => void;
  onRefreshApp: () => void;
};

function LogPanel({
  title,
  content,
  onRefresh,
}: {
  title: string;
  content: string;
  onRefresh: () => void;
}) {
  return (
    <Box borderWidth="1px" borderColor="gray.200" borderRadius="12px" p="4" bg="white">
      <HStack justify="space-between" mb="3">
        <Text fontWeight="semibold">{title}</Text>
        <Button size="sm" variant="outline" onClick={onRefresh}>
          刷新
        </Button>
      </HStack>
      <Box
        as="pre"
        bg="gray.900"
        color="gray.100"
        borderRadius="10px"
        p="3"
        minH="280px"
        maxH="420px"
        overflow="auto"
        fontSize="12px"
        lineHeight="1.5"
        whiteSpace="pre-wrap"
      >
        {content || "暂无日志"}
      </Box>
    </Box>
  );
}

function LogsPage({ typingLogText, appLogText, onRefreshTyping, onRefreshApp }: LogsPageProps) {
  return (
    <Box bg="white" borderRadius="16px" p="6" boxShadow="sm">
      <Grid templateColumns={{ base: "1fr", xl: "1fr 1fr" }} gap="4">
        <LogPanel title="Typing Log" content={typingLogText} onRefresh={onRefreshTyping} />
        <LogPanel title="App Log" content={appLogText} onRefresh={onRefreshApp} />
      </Grid>
    </Box>
  );
}

export default LogsPage;
