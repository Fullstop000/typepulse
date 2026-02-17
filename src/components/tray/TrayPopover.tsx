import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Box, Button, Grid, HStack, Spinner, Stack, Text } from "@chakra-ui/react";
import { Circle, Clock3, Keyboard, Power, SquareArrowOutUpRight } from "lucide-react";
import {
  trayActionButtonStyle,
  trayMetricCardStyle,
  trayPopoverOverlayStyle,
  trayPopoverSurfaceStyle,
} from "../../styles/glass";
import { Snapshot } from "../../types";
import { formatMs, parseRowDate } from "../../utils/stats";

const REFRESH_INTERVAL_MS = 1_500;

type TodayTotals = {
  activeMs: number;
  keyCount: number;
};

// Aggregate today's totals from snapshot rows for tray quick preview.
function buildTodayTotals(snapshot: Snapshot | null): TodayTotals {
  if (!snapshot) {
    return { activeMs: 0, keyCount: 0 };
  }
  const todayStart = new Date();
  todayStart.setHours(0, 0, 0, 0);
  const tomorrowStart = new Date(todayStart);
  tomorrowStart.setDate(tomorrowStart.getDate() + 1);

  return snapshot.rows.reduce(
    (acc, row) => {
      const rowDate = parseRowDate(row.date);
      if (rowDate < todayStart || rowDate >= tomorrowStart) {
        return acc;
      }
      acc.activeMs += row.active_typing_ms;
      acc.keyCount += row.key_count;
      return acc;
    },
    { activeMs: 0, keyCount: 0 },
  );
}

function TrayPopover() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [pendingAction, setPendingAction] = useState<"toggle" | "open" | "quit" | null>(null);

  useEffect(() => {
    let mounted = true;

    // Poll compact snapshot so tray popover stays in sync with collector state.
    const refreshSnapshot = async () => {
      try {
        const data = await invoke<Snapshot>("get_snapshot");
        if (mounted) {
          setSnapshot(data);
          setLoading(false);
        }
      } catch (error) {
        if (mounted) {
          setLoading(false);
        }
        console.error("failed to refresh tray popover snapshot", error);
      }
    };

    void refreshSnapshot();
    const timer = window.setInterval(() => {
      void refreshSnapshot();
    }, REFRESH_INTERVAL_MS);

    return () => {
      mounted = false;
      window.clearInterval(timer);
    };
  }, []);

  useEffect(() => {
    // Keep all root layers transparent so macOS vibrancy is visible through the whole panel.
    const previousHtmlBackground = document.documentElement.style.background;
    const previousHtmlBackgroundColor = document.documentElement.style.backgroundColor;
    const previousHtmlBackgroundImage = document.documentElement.style.backgroundImage;
    const previousBodyBackground = document.body.style.background;
    const previousBodyBackgroundColor = document.body.style.backgroundColor;
    const previousBodyBackgroundImage = document.body.style.backgroundImage;
    const root = document.getElementById("root");
    const previousRootBackground = root?.style.background ?? "";
    const previousRootBackgroundColor = root?.style.backgroundColor ?? "";
    const previousRootBackgroundImage = root?.style.backgroundImage ?? "";

    document.documentElement.style.background = "transparent";
    document.documentElement.style.backgroundColor = "transparent";
    document.documentElement.style.backgroundImage = "none";
    document.body.style.background = "transparent";
    document.body.style.backgroundColor = "transparent";
    document.body.style.backgroundImage = "none";
    if (root) {
      root.style.background = "transparent";
      root.style.backgroundColor = "transparent";
      root.style.backgroundImage = "none";
    }

    return () => {
      document.documentElement.style.background = previousHtmlBackground;
      document.documentElement.style.backgroundColor = previousHtmlBackgroundColor;
      document.documentElement.style.backgroundImage = previousHtmlBackgroundImage;
      document.body.style.background = previousBodyBackground;
      document.body.style.backgroundColor = previousBodyBackgroundColor;
      document.body.style.backgroundImage = previousBodyBackgroundImage;
      if (root) {
        root.style.background = previousRootBackground;
        root.style.backgroundColor = previousRootBackgroundColor;
        root.style.backgroundImage = previousRootBackgroundImage;
      }
    };
  }, []);

  useEffect(() => {
    // Request transparent window background so vibrancy and blur can be visually perceived.
    void getCurrentWindow().setBackgroundColor([0, 0, 0, 0]).catch((error) => {
      console.error("failed to set tray popover background color", error);
    });
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        void getCurrentWindow().hide();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);

  const totals = useMemo(() => buildTodayTotals(snapshot), [snapshot]);
  const isPaused = snapshot ? snapshot.paused || snapshot.auto_paused : false;
  const statusLabel = isPaused ? "暂停中" : "采集中";
  const statusColor = isPaused ? "#f59e0b" : "#10b981";

  const toggleLabel = snapshot?.auto_paused
    ? "系统自动暂停中"
    : snapshot?.paused
      ? "继续采集"
      : "暂停采集";
  const toggleTone = isPaused ? "#059669" : "#d97706";
  const toggleTextColor = isPaused ? "green.800" : "orange.800";

  const handleToggleCollecting = async () => {
    if (!snapshot || snapshot.auto_paused) {
      return;
    }
    setPendingAction("toggle");
    try {
      const data = await invoke<Snapshot>("update_paused", {
        paused: !snapshot.paused,
      });
      setSnapshot(data);
    } catch (error) {
      console.error("failed to toggle paused state from tray popover", error);
    } finally {
      setPendingAction(null);
    }
  };

  const handleOpenMain = async () => {
    setPendingAction("open");
    try {
      await invoke("show_main_panel");
      await getCurrentWindow().hide();
    } catch (error) {
      console.error("failed to open main panel from tray popover", error);
    } finally {
      setPendingAction(null);
    }
  };

  const handleQuit = async () => {
    setPendingAction("quit");
    try {
      await invoke("quit_app");
    } catch (error) {
      setPendingAction(null);
      console.error("failed to quit app from tray popover", error);
    }
  };

  return (
    <Box minH="100vh" bg="transparent" position="relative" overflow="hidden">
      <Box
        {...trayPopoverSurfaceStyle}
        position="relative"
        minH="100vh"
        p="2.5"
        borderRadius="20px"
      >
        <Box
          {...trayPopoverOverlayStyle}
          pointerEvents="none"
          position="absolute"
          inset="0"
          borderRadius="inherit"
        />
        <Stack position="relative" zIndex={1} gap="2">
          <HStack justify="space-between" align="center" mb="0.5">
            <HStack gap="2">
              <Circle size={10} fill={statusColor} color={statusColor} />
              <Text fontSize="sm" fontWeight="semibold" color="gray.900">
                TypePulse
              </Text>
            </HStack>
            <Text fontSize="xs" color="gray.700">
              {statusLabel}
            </Text>
          </HStack>

          {loading ? (
            <HStack justify="center" py="6" color="gray.700">
              <Spinner size="sm" />
              <Text fontSize="sm">加载中…</Text>
            </HStack>
          ) : (
            <Grid templateColumns="repeat(2, minmax(0, 1fr))" gap="1.5">
              <Box
                {...trayMetricCardStyle}
                py="1.5"
                px="2.25"
                borderRadius="13px"
              >
                <HStack gap="1.5" mb="1" color="gray.700" justify="center">
                  <Clock3 size={13} />
                  <Text fontSize="xs">今日打字时长</Text>
                </HStack>
                <Text
                  fontSize="2xl"
                  fontWeight="bold"
                  lineHeight="1.02"
                  color="gray.900"
                  textAlign="center"
                >
                  {formatMs(totals.activeMs)}
                </Text>
              </Box>
              <Box
                {...trayMetricCardStyle}
                py="1.5"
                px="2.25"
                borderRadius="13px"
              >
                <HStack gap="1.5" mb="1" color="gray.700" justify="center">
                  <Keyboard size={13} />
                  <Text fontSize="xs">今日按键次数</Text>
                </HStack>
                <Text
                  fontSize="2xl"
                  fontWeight="bold"
                  lineHeight="1.02"
                  color="gray.900"
                  textAlign="center"
                >
                  {totals.keyCount}
                </Text>
              </Box>
            </Grid>
          )}

          <Stack mt="0.75" gap="2">
            <Button
              {...trayActionButtonStyle}
              size="sm"
              h="43px"
              px="3.5"
              borderRadius="12px"
              onClick={handleToggleCollecting}
              loading={pendingAction === "toggle"}
              disabled={
                !snapshot ||
                snapshot.auto_paused ||
                pendingAction === "open" ||
                pendingAction === "quit"
              }
              color={toggleTextColor}
            >
              <HStack gap="2.25">
                <Circle size={8} fill={toggleTone} color={toggleTone} />
                <Text fontSize="md" fontWeight="600" lineHeight="1.1" letterSpacing="0.01em">
                  {toggleLabel}
                </Text>
              </HStack>
            </Button>
            <Grid templateColumns="1fr 1fr" gap="2">
              <Button
                {...trayActionButtonStyle}
                size="sm"
                h="43px"
                px="3.5"
                borderRadius="12px"
                onClick={handleOpenMain}
                loading={pendingAction === "open"}
                disabled={pendingAction === "toggle" || pendingAction === "quit"}
                color="gray.800"
              >
                <HStack gap="2">
                  <SquareArrowOutUpRight size={13} />
                  <Text fontSize="md" fontWeight="600" lineHeight="1.1" letterSpacing="0.01em">
                    打开面板
                  </Text>
                </HStack>
              </Button>
              <Button
                {...trayActionButtonStyle}
                size="sm"
                h="43px"
                px="3.5"
                borderRadius="12px"
                onClick={handleQuit}
                loading={pendingAction === "quit"}
                disabled={pendingAction === "toggle" || pendingAction === "open"}
                color="gray.800"
              >
                <HStack gap="2">
                  <Power size={13} />
                  <Text fontSize="md" fontWeight="600" lineHeight="1.1" letterSpacing="0.01em">
                    退出
                  </Text>
                </HStack>
              </Button>
            </Grid>
          </Stack>
        </Stack>
      </Box>
    </Box>
  );
}

export default TrayPopover;
