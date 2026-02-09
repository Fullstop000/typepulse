import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";

type SettingsPageProps = {
  paused: boolean;
  keyboardActive: boolean;
  lastError: string | null;
  onTogglePause: () => void;
};

function SettingsPage({
  paused,
  keyboardActive,
  lastError,
  onTogglePause,
}: SettingsPageProps) {
  const hasPermission = keyboardActive;
  const handleOpenPermission = async () => {
    await openUrl(
      "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent",
    );
  };
  const handleOpenDataDir = async () => {
    await invoke("open_data_dir");
  };

  return (
    <>
      <section className="card">
        <h2>采集控制</h2>
        <div className="actions">
          <button onClick={onTogglePause}>
            {paused ? "继续采集" : "暂停采集"}
          </button>
        </div>
      </section>
      <section className="card">
        <h2>系统采集授权</h2>
        <div className="status">
          <div>
            <span className="label">输入监控</span>
            <span className={hasPermission ? "ok" : "bad"}>
              {hasPermission ? "已授权" : "未授权"}
            </span>
          </div>
          <div>
            <span className="label">辅助功能</span>
            <span className={hasPermission ? "ok" : "bad"}>
              {hasPermission ? "已授权" : "未授权"}
            </span>
          </div>
        </div>
        {lastError ? <p className="error">{lastError}</p> : null}
        {!hasPermission ? (
          <div className="actions">
            <button onClick={handleOpenPermission}>前往系统设置授权</button>
          </div>
        ) : null}
      </section>
      <section className="card">
        <h2>数据存储</h2>
        <p className="subtle">数据与日志保存在本机应用数据目录。</p>
        <div className="actions">
          <button onClick={handleOpenDataDir}>前往数据目录</button>
        </div>
      </section>
    </>
  );
}

export default SettingsPage;
