import { Box, Button, Stack, Text } from "@chakra-ui/react";
import { useEffect, useState } from "react";
import { SettingSection } from "../settings/types";

type SidebarProps = {
  activeTab: "stats" | "logs" | "settings";
  activeSettingsSection: SettingSection;
  onChange: (tab: "stats" | "logs" | "settings") => void;
  onSettingsSectionChange: (section: SettingSection) => void;
};

function NavButton({
  active,
  label,
  onClick,
}: {
  active: boolean;
  label: string;
  onClick: () => void;
}) {
  return (
    <Button
      justifyContent="flex-start"
      variant={active ? "solid" : "ghost"}
      colorPalette={active ? "blue" : "gray"}
      onClick={onClick}
      w="full"
    >
      {label}
    </Button>
  );
}

function Sidebar({ activeTab, activeSettingsSection, onChange, onSettingsSectionChange }: SidebarProps) {
  const [settingsExpanded, setSettingsExpanded] = useState(activeTab === "settings");

  useEffect(() => {
    if (activeTab !== "settings") setSettingsExpanded(false);
  }, [activeTab]);

  const openSettings = () => {
    onChange("settings");
    setSettingsExpanded(true);
  };

  const handleSectionClick = (section: SettingSection) => {
    onChange("settings");
    onSettingsSectionChange(section);
    setSettingsExpanded(true);
  };

  return (
    <Box
      w={{ base: "220px", md: "240px" }}
      bg="gray.100"
      borderRightWidth="1px"
      borderColor="gray.200"
      p="4"
      minH="100vh"
      position="sticky"
      top="0"
      alignSelf="flex-start"
    >
      <Box bg="white" borderRadius="12px" p="3" mb="5">
        <Text fontWeight="bold">TypePulse</Text>
        <Text fontSize="xs" color="gray.600">Typing Analytics</Text>
      </Box>
      <Stack gap="2">
        <NavButton active={activeTab === "stats"} label="数据" onClick={() => onChange("stats")} />
        <NavButton active={activeTab === "logs"} label="日志" onClick={() => onChange("logs")} />
        <NavButton active={activeTab === "settings"} label="设置" onClick={openSettings} />
      </Stack>
      {settingsExpanded ? (
        <Stack gap="2" mt="3" pl="3" borderLeftWidth="1px" borderColor="gray.300">
          <NavButton
            active={activeTab === "settings" && activeSettingsSection === "capture"}
            label="采集控制"
            onClick={() => handleSectionClick("capture")}
          />
          <NavButton
            active={activeTab === "settings" && activeSettingsSection === "display"}
            label="展示设置"
            onClick={() => handleSectionClick("display")}
          />
          <NavButton
            active={activeTab === "settings" && activeSettingsSection === "storage"}
            label="数据存储"
            onClick={() => handleSectionClick("storage")}
          />
        </Stack>
      ) : null}
    </Box>
  );
}

export default Sidebar;
