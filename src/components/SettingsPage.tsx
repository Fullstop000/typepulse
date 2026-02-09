type SettingsPageProps = {
  paused: boolean;
  onTogglePause: () => void;
};

function SettingsPage({ paused, onTogglePause }: SettingsPageProps) {
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
        <h2>数据存储</h2>
        <p className="subtle">数据与日志保存在本机应用数据目录。</p>
      </section>
    </>
  );
}

export default SettingsPage;
