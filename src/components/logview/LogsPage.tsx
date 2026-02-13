import { Box, Button, Grid, HStack, Text } from "@chakra-ui/react";
import { glassSubtleStyle, glassSurfaceStyle } from "../../styles/glass";

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
    <Box {...glassSubtleStyle} borderRadius="12px" p="4">
      <HStack justify="space-between" mb="3">
        <Text fontWeight="semibold">{title}</Text>
        <Button
          size="sm"
          variant="ghost"
          borderWidth="1px"
          borderColor="glass.borderSoft"
          bg="rgba(255,255,255,0.56)"
          _hover={{ bg: "rgba(255,255,255,0.74)" }}
          onClick={onRefresh}
        >
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
    <Box {...glassSurfaceStyle} borderRadius="16px" p="6">
      <Grid templateColumns={{ base: "1fr", xl: "1fr 1fr" }} gap="4">
        <LogPanel title="Typing Log" content={typingLogText} onRefresh={onRefreshTyping} />
        <LogPanel title="App Log" content={appLogText} onRefresh={onRefreshApp} />
      </Grid>
    </Box>
  );
}

export default LogsPage;
