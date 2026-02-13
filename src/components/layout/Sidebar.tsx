import { Box, Button, HStack, Stack, Switch, Text } from "@chakra-ui/react";
import { ReactNode } from "react";

type SidebarProps = {
  activeTab: "stats" | "logs" | "settings";
  onChange: (tab: "stats" | "logs" | "settings") => void;
  isCollecting: boolean;
  onTogglePause: () => void;
};

type IconProps = { color: string };

function ChartIcon({ color }: IconProps) {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="1.8">
      <path d="M4 19V5" />
      <path d="M10 19v-8" />
      <path d="M16 19v-4" />
      <path d="M22 19V9" />
    </svg>
  );
}

function LogIcon({ color }: IconProps) {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="1.8">
      <path d="M5 5h14v14H5z" />
      <path d="M8 9h8" />
      <path d="M8 13h8" />
      <path d="M8 17h5" />
    </svg>
  );
}

function SettingsIcon({ color }: IconProps) {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="1.8">
      <circle cx="12" cy="12" r="3.2" />
      <path d="M19.4 15a1 1 0 0 0 .2 1.1l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1 1 0 0 0-1.1-.2 1 1 0 0 0-.6.9V20a2 2 0 1 1-4 0v-.2a1 1 0 0 0-.6-.9 1 1 0 0 0-1.1.2l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1 1 0 0 0 .2-1.1 1 1 0 0 0-.9-.6H4a2 2 0 1 1 0-4h.2a1 1 0 0 0 .9-.6 1 1 0 0 0-.2-1.1l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1 1 0 0 0 1.1.2h.1a1 1 0 0 0 .6-.9V4a2 2 0 1 1 4 0v.2a1 1 0 0 0 .6.9 1 1 0 0 0 1.1-.2l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1 1 0 0 0-.2 1.1v.1a1 1 0 0 0 .9.6H20a2 2 0 1 1 0 4h-.2a1 1 0 0 0-.9.6z" />
    </svg>
  );
}

function NavButton({
  active,
  label,
  icon,
  onClick,
}: {
  active: boolean;
  label: string;
  icon: ReactNode;
  onClick: () => void;
}) {
  const fg = active ? "#1f2328" : "#4b5563";
  return (
    <Button
      justifyContent="flex-start"
      variant="ghost"
      bg={active ? "rgba(255, 255, 255, 0.72)" : "transparent"}
      color={fg}
      _hover={{ bg: active ? "rgba(255, 255, 255, 0.78)" : "rgba(255, 255, 255, 0.48)", color: "#1f2328" }}
      fontWeight={active ? "semibold" : "medium"}
      borderRadius="10px"
      onClick={onClick}
      w="full"
      h="34px"
      px="3"
      fontSize="sm"
    >
      <HStack gap="2">
        <Box as="span" display="inline-flex" alignItems="center" justifyContent="center">
          {icon}
        </Box>
        <Text>{label}</Text>
      </HStack>
    </Button>
  );
}

function Sidebar({ activeTab, onChange, isCollecting, onTogglePause }: SidebarProps) {
  return (
    <Box
      w={{ base: "220px", md: "240px" }}
      bg="rgba(232, 234, 240, 0.78)"
      borderRightWidth="1px"
      borderColor="rgba(255, 255, 255, 0.72)"
      backdropFilter="blur(16px) saturate(1.08)"
      css={{ WebkitBackdropFilter: "blur(16px) saturate(1.08)" }}
      boxShadow="inset -1px 0 0 rgba(255,255,255,0.45)"
      p="4"
      minH="100vh"
      position="sticky"
      top="0"
      alignSelf="flex-start"
      display="flex"
      flexDirection="column"
    >
      <Box borderRadius="12px" p="2" mb="4">
        <Text fontWeight="semibold" fontSize="sm" color="#4b5563">TypePulse</Text>
        <Text fontSize="xs" color="#7b8390">Typing Analytics</Text>
      </Box>
      <Stack gap="1">
        <NavButton active={activeTab === "stats"} label="数据" icon={<ChartIcon color={activeTab === "stats" ? "#1f2328" : "#6b7280"} />} onClick={() => onChange("stats")} />
        <NavButton active={activeTab === "logs"} label="日志" icon={<LogIcon color={activeTab === "logs" ? "#1f2328" : "#6b7280"} />} onClick={() => onChange("logs")} />
        <NavButton active={activeTab === "settings"} label="设置" icon={<SettingsIcon color={activeTab === "settings" ? "#1f2328" : "#6b7280"} />} onClick={() => onChange("settings")} />
      </Stack>
      <Box mt="auto" pt="4">
        <HStack
          px="2"
          py="2"
          borderRadius="10px"
          bg="glass.subtle"
          borderWidth="1px"
          borderColor="glass.borderSoft"
          backdropFilter="blur(10px) saturate(1.04)"
          css={{ WebkitBackdropFilter: "blur(10px) saturate(1.04)" }}
          justify="space-between"
        >
          <HStack gap="1.5">
            <Box
              w="8px"
              h="8px"
              borderRadius="full"
              bg={isCollecting ? "#22c55e" : "#facc15"}
              borderWidth="1px"
              borderColor={isCollecting ? "#16a34a" : "#eab308"}
            />
            <Text fontSize="xs" color="#6b7280">
              {isCollecting ? "采集中" : "暂停采集"}
            </Text>
          </HStack>
          <Switch.Root checked={isCollecting} onCheckedChange={onTogglePause}>
            <Switch.HiddenInput />
            <Switch.Control />
          </Switch.Root>
        </HStack>
      </Box>
    </Box>
  );
}

export default Sidebar;
