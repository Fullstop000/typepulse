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
            data-tooltip="按键间隔不超过5秒的时间，加起来就是打字时长"
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
            data-tooltip="你一共按了多少次键"
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
            data-tooltip="两次按键隔了超过5秒，就算开始了一次新会话"
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
