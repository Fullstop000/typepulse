import { useEffect, useState } from "react";
import { SettingSection } from "./settings/types";

type SidebarProps = {
  activeTab: "stats" | "logs" | "settings";
  activeSettingsSection: SettingSection;
  onChange: (tab: "stats" | "logs" | "settings") => void;
  onSettingsSectionChange: (section: SettingSection) => void;
};

function Sidebar({
  activeTab,
  activeSettingsSection,
  onChange,
  onSettingsSectionChange,
}: SidebarProps) {
  const [settingsExpanded, setSettingsExpanded] = useState(activeTab === "settings");

  useEffect(() => {
    if (activeTab !== "settings") {
      setSettingsExpanded(false);
    }
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
    <aside className="sidebar">
      <div className="sidebar-brand">
        <div className="brand-mark">◎</div>
        <div className="brand-text">
          <div className="brand-name">TypePulse</div>
          <div className="brand-subtle">Typing Analytics</div>
        </div>
      </div>
      <nav className="nav">
        <button
          className={activeTab === "stats" ? "nav-item active" : "nav-item"}
          onClick={() => onChange("stats")}
        >
          数据
        </button>
        <button
          className={activeTab === "logs" ? "nav-item active" : "nav-item"}
          onClick={() => onChange("logs")}
        >
          日志
        </button>
        <button
          className={activeTab === "settings" ? "nav-item active" : "nav-item"}
          onClick={openSettings}
        >
          设置
        </button>
        {settingsExpanded ? (
          <div className="sub-nav">
            <button
              className={
                activeTab === "settings" && activeSettingsSection === "capture"
                  ? "sub-nav-item active"
                  : "sub-nav-item"
              }
              onClick={() => handleSectionClick("capture")}
            >
              采集控制
            </button>
            <button
              className={
                activeTab === "settings" && activeSettingsSection === "display"
                  ? "sub-nav-item active"
                  : "sub-nav-item"
              }
              onClick={() => handleSectionClick("display")}
            >
              展示设置
            </button>
            <button
              className={
                activeTab === "settings" && activeSettingsSection === "storage"
                  ? "sub-nav-item active"
                  : "sub-nav-item"
              }
              onClick={() => handleSectionClick("storage")}
            >
              数据存储
            </button>
          </div>
        ) : null}
      </nav>
    </aside>
  );
}

export default Sidebar;
