import { Snapshot } from "../types";

type StatusCardProps = {
  snapshot: Snapshot;
};

function StatusCard({ snapshot }: StatusCardProps) {
  return (
    <section className="card">
      <div className="status">
        <div>
          <span className="label">键盘监听</span>
          <span className={snapshot.keyboard_active ? "ok" : "bad"}>
            {snapshot.keyboard_active ? "已启用" : "未启用"}
          </span>
        </div>
        <div>
          <span className="label">采集状态</span>
          <span className={snapshot.paused ? "bad" : "ok"}>
            {snapshot.paused ? "已暂停" : "运行中"}
          </span>
        </div>
      </div>
      {snapshot.last_error ? (
        <p className="error">{snapshot.last_error}</p>
      ) : null}
    </section>
  );
}

export default StatusCard;
