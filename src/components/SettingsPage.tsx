import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";

type SettingsPageProps = {
  paused: boolean;
  keyboardActive: boolean;
  ignoreKeyCombos: boolean;
  lastError: string | null;
  onTogglePause: () => void;
  onToggleIgnoreKeyCombos: () => void;
};

function SettingsPage({
  paused,
  keyboardActive,
  ignoreKeyCombos,
  lastError,
  onTogglePause,
  onToggleIgnoreKeyCombos,
}: SettingsPageProps) {
  const [dataSize, setDataSize] = useState<number | null>(null);
  const hasPermission = keyboardActive;
  const handleOpenPermission = async () => {
    await openUrl(
      "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent",
    );
  };
  const handleOpenDataDir = async () => {
    await invoke("open_data_dir");
  };
  const formatBytes = (bytes: number) => {
    if (bytes < 1024) {
      return `${bytes} B`;
    }
    const units = ["KB", "MB", "GB", "TB"];
    let size = bytes;
    let index = -1;
    while (size >= 1024 && index < units.length - 1) {
      size /= 1024;
      index += 1;
    }
    const precision = size >= 10 || index === 0 ? 0 : 1;
    return `${size.toFixed(precision)} ${units[index]}`;
  };

  useEffect(() => {
    let mounted = true;
    invoke<number>("get_data_dir_size")
      .then((size) => {
        if (mounted) {
          setDataSize(size);
        }
      })
      .catch(() => {
        if (mounted) {
          setDataSize(null);
        }
      });
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <>
      <section className="card">
        <h2>采集控制</h2>
        <div className="actions">
          <button onClick={onTogglePause}>
            {paused ? "继续采集" : "暂停采集"}
          </button>
        </div>
        <div className="setting-row">
          <div>
            <span className="label">忽略组合键</span>
            <p className="subtle">开启后不记录 Ctrl/Alt/Fn/Shift/Cmd + 任意键。</p>
          </div>
          <button className="secondary" onClick={onToggleIgnoreKeyCombos}>
            {ignoreKeyCombos ? "已开启" : "已关闭"}
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
        {dataSize !== null ? (
          <p className="subtle">已用空间：{formatBytes(dataSize)}</p>
        ) : null}
        <div className="actions">
          <button onClick={handleOpenDataDir}>前往数据目录</button>
        </div>
      </section>
    </>
  );
}

export default SettingsPage;
