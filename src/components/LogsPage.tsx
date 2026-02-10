type LogsPageProps = {
  typingLogText: string;
  appLogText: string;
  onRefreshTyping: () => void;
  onRefreshApp: () => void;
};

function LogsPage({
  typingLogText,
  appLogText,
  onRefreshTyping,
  onRefreshApp,
}: LogsPageProps) {
  return (
    <section className="card">
      <div className="log-grid">
        <h2>Typing Log</h2>
        <div className="log-panel">
          <div className="row">
            <button onClick={onRefreshTyping} className="secondary">
              刷新
            </button>
          </div>
          <pre className="log-content">
            {typingLogText ? typingLogText : "暂无日志"}
          </pre>
        </div>
        <div className="log-panel">
          <h2>App Log</h2>
          <div className="row">
            <button onClick={onRefreshApp} className="secondary">
              刷新
            </button>
          </div>
          <pre className="log-content">
            {appLogText ? appLogText : "暂无日志"}
          </pre>
        </div>
      </div>
    </section>
  );
}

export default LogsPage;
