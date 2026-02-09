type SidebarProps = {
  activeTab: "stats" | "logs" | "settings";
  onChange: (tab: "stats" | "logs" | "settings") => void;
};

function Sidebar({ activeTab, onChange }: SidebarProps) {
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
          onClick={() => onChange("settings")}
        >
          设置
        </button>
      </nav>
    </aside>
  );
}

export default Sidebar;
