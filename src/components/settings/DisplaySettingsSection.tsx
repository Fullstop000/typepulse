import { MenuBarDisplayMode } from "../../types";
import { useSettingsContext } from "./SettingsContext";

function DisplaySettingsSection() {
  const { snapshot, updateTrayDisplayMode } = useSettingsContext();

  return (
    <section className="card">
      <h2>展示设置</h2>
      <div className="setting-row">
        <div>
          <span className="setting-title">菜单栏显示模式</span>
          <p className="setting-desc">控制菜单栏小组件展示为图标、数字或图标+数字。</p>
        </div>
        <select
          className="secondary"
          value={snapshot.tray_display_mode}
          onChange={(event) =>
            updateTrayDisplayMode(event.target.value as MenuBarDisplayMode)
          }
        >
          <option value="icon_only">仅图标</option>
          <option value="text_only">仅数字</option>
          <option value="icon_text">图标 + 数字</option>
        </select>
      </div>
    </section>
  );
}

export default DisplaySettingsSection;
