import { Totals } from "../types";
import { formatMs } from "../utils/stats";

type MetricsGridProps = {
  totals: Totals;
};

function MetricsGrid({ totals }: MetricsGridProps) {
  return (
    <section className="card grid">
      <div>
        <div className="label label-row">
          打字时长
          <span
            className="info-icon"
            data-tooltip="相邻按键间隔≤5秒的时间差累加"
            tabIndex={0}
          >
            ⓘ
          </span>
        </div>
        <div className="metric">{formatMs(totals.active)}</div>
      </div>
      <div>
        <div className="label label-row">
          按键次数
          <span
            className="info-icon"
            data-tooltip="按键事件次数"
            tabIndex={0}
          >
            ⓘ
          </span>
        </div>
        <div className="metric">{totals.keys}</div>
      </div>
      <div>
        <div className="label label-row">
          会话次数
          <span
            className="info-icon"
            data-tooltip="按键间隔>5秒记为新会话"
            tabIndex={0}
          >
            ⓘ
          </span>
        </div>
        <div className="metric">{totals.sessions}</div>
      </div>
    </section>
  );
}

export default MetricsGrid;
