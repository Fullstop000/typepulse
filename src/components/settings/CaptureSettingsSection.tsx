import { openUrl } from "@tauri-apps/plugin-opener";
import { Badge, Box, Button, HStack, Stack, Text } from "@chakra-ui/react";
import { useMemo, useState } from "react";
import { useSettingsContext } from "./SettingsContext";

function CaptureSettingsSection() {
  const {
    snapshot,
    togglePause,
    toggleIgnoreKeyCombos,
    addAppExclusion,
    removeAppExclusion,
    loadRunningApps,
    dismissOnePasswordSuggestion,
    acceptOnePasswordSuggestion,
  } = useSettingsContext();

  const [runningAppsOpen, setRunningAppsOpen] = useState(false);
  const [runningApps, setRunningApps] = useState<{ bundle_id: string; name: string }[]>([]);
  const [loadingRunningApps, setLoadingRunningApps] = useState(false);

  const hasPermission = snapshot.keyboard_active;

  const excludedSet = useMemo(
    () => new Set(snapshot.excluded_bundle_ids.map((item) => item.toLowerCase())),
    [snapshot.excluded_bundle_ids],
  );

  const statusMessage = useMemo(() => {
    if (snapshot.paused) return "当前为手动暂停。";
    if (snapshot.auto_paused && snapshot.auto_pause_reason === "blacklist") {
      return "当前焦点在忽略应用中，采集已自动暂停。";
    }
    if (snapshot.auto_paused && snapshot.auto_pause_reason === "secure_input") {
      return "检测到系统安全输入模式（密码框），采集已自动暂停。";
    }
    return "正在采集。";
  }, [snapshot.auto_pause_reason, snapshot.auto_paused, snapshot.paused]);

  const handleOpenPermission = async () => {
    await openUrl("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent");
  };

  const handleOpenRunningApps = async () => {
    setRunningAppsOpen(true);
    setLoadingRunningApps(true);
    try {
      const apps = await loadRunningApps();
      setRunningApps(apps);
    } finally {
      setLoadingRunningApps(false);
    }
  };

  return (
    <>
      <Box bg="white" borderRadius="16px" p="6" boxShadow="sm">
        <Text fontSize="xl" fontWeight="semibold" mb="2">采集控制</Text>
        <Text fontSize="sm" color="gray.600" mb="4">{statusMessage}</Text>

        <Button onClick={togglePause} mb="4">{snapshot.paused ? "继续采集" : "暂停采集"}</Button>

        <Stack gap="4">
          <HStack justify="space-between" align="start" flexWrap="wrap" gap="3">
            <Box>
              <Text fontWeight="semibold">忽略组合键</Text>
              <Text fontSize="sm" color="gray.600">开启后不记录 Ctrl/Alt/Fn/Shift/Cmd + 任意键。</Text>
            </Box>
            <Button variant="outline" onClick={toggleIgnoreKeyCombos}>
              {snapshot.ignore_key_combos ? "已开启" : "已关闭"}
            </Button>
          </HStack>

          <HStack justify="space-between" align="start" flexWrap="wrap" gap="3">
            <Box>
              <Text fontWeight="semibold">密码输入保护</Text>
              <Text fontSize="sm" color="gray.600">检测到密码输入框时，自动忽略输入内容，不会写入统计。</Text>
            </Box>
            <Badge colorPalette="green">已启用</Badge>
          </HStack>

          <HStack justify="space-between" align="start" flexWrap="wrap" gap="3">
            <Box>
              <Text fontWeight="semibold">系统采集授权</Text>
              <Text fontSize="sm" color="gray.600">检查输入监控与辅助功能授权状态。</Text>
            </Box>
            {!hasPermission ? (
              <Button variant="outline" onClick={handleOpenPermission}>前往授权</Button>
            ) : (
              <Badge colorPalette="green">已授权</Badge>
            )}
          </HStack>

          <HStack justify="space-between" align="start" flexWrap="wrap" gap="3">
            <Box>
              <Text fontWeight="semibold">忽略应用</Text>
              <Text fontSize="sm" color="gray.600">管理已忽略应用列表。</Text>
            </Box>
            <Button variant="outline" onClick={handleOpenRunningApps}>+ 添加应用</Button>
          </HStack>
        </Stack>

        {snapshot.one_password_suggestion_pending ? (
          <Box mt="4" borderWidth="1px" borderColor="blue.200" bg="blue.50" borderRadius="10px" p="4">
            <Text mb="3">检测到你安装了 1Password，是否加入忽略列表？</Text>
            <HStack>
              <Button size="sm" onClick={acceptOnePasswordSuggestion}>加入忽略列表</Button>
              <Button size="sm" variant="outline" onClick={dismissOnePasswordSuggestion}>暂不</Button>
            </HStack>
          </Box>
        ) : null}

        <Box mt="5" borderWidth="1px" borderColor="gray.200" borderRadius="12px" overflow="hidden">
          <HStack px="4" py="3" bg="gray.50" fontWeight="semibold" fontSize="sm" justify="space-between">
            <Text flex="1">Bundle ID</Text>
            <Text flex="0 0 auto">操作</Text>
          </HStack>
          {snapshot.excluded_bundle_ids.length === 0 ? (
            <Text px="4" py="6" color="gray.500" textAlign="center">暂无忽略应用</Text>
          ) : (
            snapshot.excluded_bundle_ids.map((bundleId) => (
              <HStack key={bundleId} px="4" py="3" borderTopWidth="1px" borderColor="gray.100" justify="space-between" gap="3">
                <Text fontFamily="mono" fontSize="sm" truncate title={bundleId}>
                  {bundleId}
                </Text>
                <Button size="sm" variant="outline" onClick={() => removeAppExclusion(bundleId)}>移除</Button>
              </HStack>
            ))
          )}
        </Box>

        <HStack gap="6" mt="5" flexWrap="wrap">
          <HStack>
            <Text fontSize="sm">输入监控</Text>
            <Badge colorPalette={hasPermission ? "green" : "red"}>{hasPermission ? "已授权" : "未授权"}</Badge>
          </HStack>
          <HStack>
            <Text fontSize="sm">辅助功能</Text>
            <Badge colorPalette={hasPermission ? "green" : "red"}>{hasPermission ? "已授权" : "未授权"}</Badge>
          </HStack>
        </HStack>

        {snapshot.last_error ? (
          <Text mt="4" color="red.600" fontSize="sm">{snapshot.last_error}</Text>
        ) : null}
      </Box>

      {runningAppsOpen ? (
        <Box
          position="fixed"
          inset="0"
          bg="blackAlpha.600"
          display="flex"
          alignItems="center"
          justifyContent="center"
          zIndex={1000}
          onClick={() => setRunningAppsOpen(false)}
        >
          <Box
            bg="white"
            borderRadius="12px"
            p="5"
            w="min(900px, 90vw)"
            maxH="80vh"
            overflow="auto"
            onClick={(event) => event.stopPropagation()}
          >
            <HStack justify="space-between" mb="4">
              <Text fontSize="lg" fontWeight="semibold">正在运行的应用</Text>
              <Button size="sm" variant="outline" onClick={() => setRunningAppsOpen(false)}>关闭</Button>
            </HStack>
            {loadingRunningApps ? (
              <Text color="gray.600">加载中…</Text>
            ) : (
              <Box borderWidth="1px" borderColor="gray.200" borderRadius="12px" overflow="hidden">
                <HStack px="4" py="3" bg="gray.50" fontWeight="semibold" fontSize="sm" justify="space-between" gap="3">
                  <Text flex="1">应用</Text>
                  <Text flex="1">Bundle ID</Text>
                  <Text flex="0 0 auto">操作</Text>
                </HStack>
                {runningApps.length === 0 ? (
                  <Text px="4" py="6" color="gray.500" textAlign="center">未检测到可用应用</Text>
                ) : (
                  runningApps.map((app) => {
                    const selected = excludedSet.has(app.bundle_id.toLowerCase());
                    return (
                      <HStack key={app.bundle_id} px="4" py="3" borderTopWidth="1px" borderColor="gray.100" justify="space-between" gap="3">
                        <Text flex="1" truncate title={app.name}>{app.name}</Text>
                        <Text flex="1" fontFamily="mono" fontSize="sm" truncate title={app.bundle_id}>{app.bundle_id}</Text>
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() =>
                            selected
                              ? removeAppExclusion(app.bundle_id)
                              : addAppExclusion(app.bundle_id)
                          }
                        >
                          {selected ? "已忽略" : "添加"}
                        </Button>
                      </HStack>
                    );
                  })
                )}
              </Box>
            )}
          </Box>
        </Box>
      ) : null}
    </>
  );
}

export default CaptureSettingsSection;
