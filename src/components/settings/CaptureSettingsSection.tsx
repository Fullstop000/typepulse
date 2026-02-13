import { openUrl } from "@tauri-apps/plugin-opener";
import { Badge, Box, Button, HStack, Stack, Switch, Text } from "@chakra-ui/react";
import { useMemo, useState } from "react";
import { glassSubtleStyle, glassSurfaceStyle } from "../../styles/glass";
import { useSettingsContext } from "./SettingsContext";

function CaptureSettingsSection() {
  const {
    snapshot,
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
      <Box {...glassSurfaceStyle} borderRadius="12px" overflow="hidden">
        <Box px="5" py="4" borderBottomWidth="1px" borderColor="glass.borderSoft">
          <Text fontSize="lg" fontWeight="semibold" color="#111827">General</Text>
        </Box>

        <Stack gap="0">
          <HStack justify="space-between" align="center" flexWrap="wrap" gap="3" px="5" py="4" borderBottomWidth="1px" borderColor="glass.borderSoft">
            <Box>
              <Text fontWeight="medium" color="#111827">忽略组合键</Text>
              <Text fontSize="sm" color="#6b7280">开启后不记录 Ctrl/Alt/Fn/Shift/Cmd + 任意键。</Text>
            </Box>
            <Switch.Root checked={snapshot.ignore_key_combos} onCheckedChange={toggleIgnoreKeyCombos}>
              <Switch.HiddenInput />
              <Switch.Control />
            </Switch.Root>
          </HStack>

          <HStack justify="space-between" align="start" flexWrap="wrap" gap="3" px="5" py="4" borderBottomWidth="1px" borderColor="glass.borderSoft">
            <Box>
              <Text fontWeight="medium" color="#111827">密码输入保护</Text>
              <Text fontSize="sm" color="#6b7280">检测到密码输入框时，自动忽略输入内容，不会写入统计。</Text>
            </Box>
            <Badge bg="#dff2e2" color="#166534">已启用</Badge>
          </HStack>

          <HStack justify="space-between" align="start" flexWrap="wrap" gap="3" px="5" py="4" borderBottomWidth="1px" borderColor="glass.borderSoft">
            <Box>
              <Text fontWeight="medium" color="#111827">系统采集授权</Text>
              <Text fontSize="sm" color="#6b7280">检查输入监控与辅助功能两项授权状态。</Text>
            </Box>
            {!hasPermission ? (
              <Button
                variant="ghost"
                borderWidth="1px"
                borderColor="glass.borderSoft"
                bg="rgba(255,255,255,0.56)"
                _hover={{ bg: "rgba(255,255,255,0.74)" }}
                onClick={handleOpenPermission}
              >
                前往授权
              </Button>
            ) : (
              <Badge bg="#dff2e2" color="#166534">已授权</Badge>
            )}
          </HStack>

          <HStack justify="space-between" align="start" flexWrap="wrap" gap="3" px="5" py="4">
            <Box>
              <Text fontWeight="medium" color="#111827">忽略应用</Text>
              <Text fontSize="sm" color="#6b7280">管理已忽略应用列表。</Text>
            </Box>
            <Button
              variant="ghost"
              borderWidth="1px"
              borderColor="glass.borderSoft"
              bg="rgba(255,255,255,0.56)"
              _hover={{ bg: "rgba(255,255,255,0.74)" }}
              onClick={handleOpenRunningApps}
            >
              + 添加应用
            </Button>
          </HStack>
        </Stack>

        {snapshot.one_password_suggestion_pending ? (
          <Box m="5" {...glassSubtleStyle} borderRadius="10px" p="4">
            <Text mb="3">检测到你安装了 1Password，是否加入忽略列表？</Text>
            <HStack>
              <Button
                size="sm"
                variant="ghost"
                borderWidth="1px"
                borderColor="glass.borderSoft"
                bg="rgba(255,255,255,0.62)"
                _hover={{ bg: "rgba(255,255,255,0.8)" }}
                onClick={acceptOnePasswordSuggestion}
              >
                加入忽略列表
              </Button>
              <Button
                size="sm"
                variant="ghost"
                borderWidth="1px"
                borderColor="glass.borderSoft"
                bg="rgba(255,255,255,0.5)"
                _hover={{ bg: "rgba(255,255,255,0.7)" }}
                onClick={dismissOnePasswordSuggestion}
              >
                暂不
              </Button>
            </HStack>
          </Box>
        ) : null}

        <Box m="5" {...glassSubtleStyle} borderRadius="12px" overflow="hidden">
          <HStack px="4" py="3" bg="rgba(255,255,255,0.46)" fontWeight="semibold" fontSize="sm" justify="space-between">
            <Text flex="1">Bundle ID</Text>
            <Text flex="0 0 auto">操作</Text>
          </HStack>
          {snapshot.excluded_bundle_ids.length === 0 ? (
            <Text px="4" py="6" color="#8b939f" textAlign="center">暂无忽略应用</Text>
          ) : (
            snapshot.excluded_bundle_ids.map((bundleId) => (
              <HStack key={bundleId} px="4" py="3" borderTopWidth="1px" borderColor="glass.borderSoft" justify="space-between" gap="3">
                <Text fontFamily="mono" fontSize="sm" truncate title={bundleId}>
                  {bundleId}
                </Text>
                <Button
                  size="sm"
                  variant="ghost"
                  borderWidth="1px"
                  borderColor="glass.borderSoft"
                  bg="rgba(255,255,255,0.56)"
                  _hover={{ bg: "rgba(255,255,255,0.74)" }}
                  onClick={() => removeAppExclusion(bundleId)}
                >
                  移除
                </Button>
              </HStack>
            ))
          )}
        </Box>

        {snapshot.last_error ? (
          <Text px="5" pb="5" color="red.600" fontSize="sm">{snapshot.last_error}</Text>
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
            {...glassSurfaceStyle}
            borderRadius="12px"
            p="5"
            w="min(900px, 90vw)"
            maxH="80vh"
            overflow="auto"
            onClick={(event) => event.stopPropagation()}
          >
            <HStack justify="space-between" mb="4">
              <Text fontSize="lg" fontWeight="semibold">正在运行的应用</Text>
              <Button
                size="sm"
                variant="ghost"
                borderWidth="1px"
                borderColor="glass.borderSoft"
                bg="rgba(255,255,255,0.56)"
                _hover={{ bg: "rgba(255,255,255,0.74)" }}
                onClick={() => setRunningAppsOpen(false)}
              >
                关闭
              </Button>
            </HStack>
            {loadingRunningApps ? (
              <Text color="#6b7280">加载中…</Text>
            ) : (
              <Box {...glassSubtleStyle} borderRadius="12px" overflow="hidden">
                <HStack px="4" py="3" bg="rgba(255,255,255,0.46)" fontWeight="semibold" fontSize="sm" justify="space-between" gap="3">
                  <Text flex="1">应用</Text>
                  <Text flex="1">Bundle ID</Text>
                  <Text flex="0 0 auto">操作</Text>
                </HStack>
                {runningApps.length === 0 ? (
                  <Text px="4" py="6" color="#8b939f" textAlign="center">未检测到可用应用</Text>
                ) : (
                  runningApps.map((app) => {
                    const selected = excludedSet.has(app.bundle_id.toLowerCase());
                    return (
                      <HStack key={app.bundle_id} px="4" py="3" borderTopWidth="1px" borderColor="glass.borderSoft" justify="space-between" gap="3">
                        <Text flex="1" truncate title={app.name}>{app.name}</Text>
                        <Text flex="1" fontFamily="mono" fontSize="sm" truncate title={app.bundle_id}>{app.bundle_id}</Text>
                        <Button
                          size="sm"
                          variant="ghost"
                          borderWidth="1px"
                          borderColor="glass.borderSoft"
                          bg="rgba(255,255,255,0.56)"
                          _hover={{ bg: "rgba(255,255,255,0.74)" }}
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
