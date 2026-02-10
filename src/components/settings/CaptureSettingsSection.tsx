import { useMemo, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
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
  const [runningApps, setRunningApps] = useState<
    { bundle_id: string; name: string }[]
  >([]);
  const [loadingRunningApps, setLoadingRunningApps] = useState(false);
  const hasPermission = snapshot.keyboard_active;

  const excludedSet = useMemo(() => {
    return new Set(snapshot.excluded_bundle_ids.map((item) => item.toLowerCase()));
  }, [snapshot.excluded_bundle_ids]);

  const statusMessage = useMemo(() => {
    if (snapshot.paused) {
      return "当前为手动暂停。";
    }
    if (snapshot.auto_paused && snapshot.auto_pause_reason === "blacklist") {
      return "当前焦点在忽略应用中，采集已自动暂停。";
    }
    if (snapshot.auto_paused && snapshot.auto_pause_reason === "secure_input") {
      return "检测到系统安全输入模式（密码框），采集已自动暂停。";
    }
    return "正在采集。";
  }, [snapshot.auto_pause_reason, snapshot.auto_paused, snapshot.paused]);

  const handleOpenPermission = async () => {
    await openUrl(
      "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent",
    );
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
      <section className="card">
        <h2>采集控制</h2>
        <p className="subtle">{statusMessage}</p>
        <div className="actions">
          <button onClick={togglePause}>{snapshot.paused ? "继续采集" : "暂停采集"}</button>
        </div>
        <div className="setting-row">
          <div>
            <span className="setting-title">忽略组合键</span>
            <p className="setting-desc">开启后不记录 Ctrl/Alt/Fn/Shift/Cmd + 任意键。</p>
          </div>
          <button className="secondary" onClick={toggleIgnoreKeyCombos}>
            {snapshot.ignore_key_combos ? "已开启" : "已关闭"}
          </button>
        </div>
        <div className="setting-row">
          <div>
            <span className="setting-title">密码输入保护</span>
            <p className="setting-desc">
              检测到密码输入框时，自动忽略输入内容，不会写入统计。
            </p>
          </div>
          <span className="ok">已启用</span>
        </div>
        <div className="setting-row">
          <div>
            <span className="setting-title">系统采集授权</span>
            <p className="setting-desc">检查输入监控与辅助功能授权状态。</p>
          </div>
          {!hasPermission ? (
            <button className="secondary" onClick={handleOpenPermission}>
              前往授权
            </button>
          ) : (
            <span className="ok">已授权</span>
          )}
        </div>
        <div className="setting-row">
          <div>
            <span className="setting-title">忽略应用</span>
            <p className="setting-desc">管理已忽略应用列表。</p>
          </div>
          <button className="secondary" onClick={handleOpenRunningApps}>
            + 添加应用
          </button>
        </div>
        {snapshot.one_password_suggestion_pending ? (
          <div className="suggestion-card">
            <p className="label">检测到你安装了 1Password，是否加入忽略列表？</p>
            <div className="actions">
              <button onClick={acceptOnePasswordSuggestion}>加入忽略列表</button>
              <button className="secondary" onClick={dismissOnePasswordSuggestion}>
                暂不
              </button>
            </div>
          </div>
        ) : null}
        <div className="table exclusion-table">
          <div className="table-header exclusion-row">
            <span>Bundle ID</span>
            <span>操作</span>
          </div>
          {snapshot.excluded_bundle_ids.length === 0 ? (
            <div className="table-empty">暂无忽略应用</div>
          ) : (
            snapshot.excluded_bundle_ids.map((bundleId) => (
              <div key={bundleId} className="table-row exclusion-row">
                <span className="mono truncate" title={bundleId}>
                  {bundleId}
                </span>
                <button className="secondary" onClick={() => removeAppExclusion(bundleId)}>
                  移除
                </button>
              </div>
            ))
          )}
        </div>
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
        {snapshot.last_error ? <p className="error">{snapshot.last_error}</p> : null}
      </section>

      {runningAppsOpen ? (
        <div className="modal-overlay" onClick={() => setRunningAppsOpen(false)}>
          <section className="card modal-panel" onClick={(event) => event.stopPropagation()}>
            <div className="row running-apps-header">
              <h2>正在运行的应用</h2>
              <button className="secondary" onClick={() => setRunningAppsOpen(false)}>
                关闭
              </button>
            </div>
            <div className="modal-body">
              {loadingRunningApps ? (
                <p className="subtle">加载中…</p>
              ) : (
                <div className="table exclusion-table">
                  <div className="table-header running-app-row">
                    <span>应用</span>
                    <span>Bundle ID</span>
                    <span>操作</span>
                  </div>
                  {runningApps.length === 0 ? (
                    <div className="table-empty">未检测到可用应用</div>
                  ) : (
                    runningApps.map((app) => {
                      const selected = excludedSet.has(app.bundle_id.toLowerCase());
                      return (
                        <div key={app.bundle_id} className="table-row running-app-row">
                          <span className="truncate" title={app.name}>
                            {app.name}
                          </span>
                          <span className="mono truncate" title={app.bundle_id}>
                            {app.bundle_id}
                          </span>
                          <button
                            className="secondary"
                            onClick={() =>
                              selected
                                ? removeAppExclusion(app.bundle_id)
                                : addAppExclusion(app.bundle_id)
                            }
                          >
                            {selected ? "已忽略" : "添加"}
                          </button>
                        </div>
                      );
                    })
                  )}
                </div>
              )}
            </div>
          </section>
        </div>
      ) : null}
    </>
  );
}

export default CaptureSettingsSection;
