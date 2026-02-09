import { TrendGranularity, TrendPoint } from "../types";

type TrendChartProps = {
  series: TrendPoint[];
  granularity: TrendGranularity;
  onGranularityChange: (value: TrendGranularity) => void;
};

function TrendChart({
  series,
  granularity,
  onGranularityChange,
}: TrendChartProps) {
  return (
    <section className="card">
      <div className="chart-header">
        <h2>输入活跃度趋势</h2>
        <div className="row">
          <button
            onClick={() => onGranularityChange("1m")}
            className={granularity === "1m" ? "tab active" : "tab"}
          >
            1分钟
          </button>
          <button
            onClick={() => onGranularityChange("5m")}
            className={granularity === "5m" ? "tab active" : "tab"}
          >
            5分钟
          </button>
          <button
            onClick={() => onGranularityChange("1h")}
            className={granularity === "1h" ? "tab active" : "tab"}
          >
            1小时
          </button>
          <button
            onClick={() => onGranularityChange("1d")}
            className={granularity === "1d" ? "tab active" : "tab"}
          >
            1天
          </button>
        </div>
      </div>
      <div className="chart">
        {series.length === 0 ? (
          <div className="table-empty">暂无数据</div>
        ) : (
          <svg
            className="chart-svg"
            viewBox="0 0 640 180"
            preserveAspectRatio="none"
          >
            {(() => {
              const paddingLeft = 44;
              const paddingRight = 16;
              const paddingTop = 16;
              const paddingBottom = 28;
              const width = 640 - paddingLeft - paddingRight;
              const height = 180 - paddingTop - paddingBottom;
              const maxValue = Math.max(1, ...series.map((d) => d.value));
              const points = series.map((d, index) => {
                const x =
                  paddingLeft +
                  (index / Math.max(1, series.length - 1)) * width;
                const y = paddingTop + (1 - d.value / maxValue) * height;
                return `${x},${y}`;
              });
              const startLabel = series[0]?.label ?? "";
              const midIndex = Math.floor(series.length / 2);
              const midLabel = series[midIndex]?.label ?? "";
              const endLabel = series[series.length - 1]?.label ?? "";
              const axisY = paddingTop + height;
              return (
                <>
                  <line
                    className="chart-axis"
                    x1={paddingLeft}
                    y1={axisY}
                    x2={paddingLeft + width}
                    y2={axisY}
                  />
                  <line
                    className="chart-axis"
                    x1={paddingLeft}
                    y1={paddingTop}
                    x2={paddingLeft}
                    y2={axisY}
                  />
                  <polyline className="chart-line" points={points.join(" ")} />
                  <text
                    className="chart-label"
                    x={paddingLeft}
                    y={axisY + 18}
                    textAnchor="start"
                  >
                    {startLabel}
                  </text>
                  <text
                    className="chart-label"
                    x={paddingLeft + width / 2}
                    y={axisY + 18}
                    textAnchor="middle"
                  >
                    {midLabel}
                  </text>
                  <text
                    className="chart-label"
                    x={paddingLeft + width}
                    y={axisY + 18}
                    textAnchor="end"
                  >
                    {endLabel}
                  </text>
                  <text
                    className="chart-label"
                    x={paddingLeft - 8}
                    y={paddingTop + 4}
                    textAnchor="end"
                  >
                    {maxValue}s
                  </text>
                  <text
                    className="chart-label"
                    x={paddingLeft - 8}
                    y={axisY}
                    textAnchor="end"
                  >
                    0s
                  </text>
                </>
              );
            })()}
          </svg>
        )}
      </div>
    </section>
  );
}

export default TrendChart;
