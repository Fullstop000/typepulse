import { useEffect, useMemo, useRef } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { TrendGranularity, TrendSeries } from "../types";

type TrendChartProps = {
  series: TrendSeries;
  granularity: TrendGranularity;
  onGranularityChange: (value: TrendGranularity) => void;
};

function TrendChart({
  series,
  granularity,
  onGranularityChange,
}: TrendChartProps) {
  const chartRef = useRef<HTMLDivElement | null>(null);
  const plotRef = useRef<uPlot | null>(null);
  const hasData = series.timestamps.length > 0;
  const data = useMemo<uPlot.AlignedData>(() => {
    return [
      series.timestamps,
      series.activeSeconds,
      series.keyCounts,
    ] as uPlot.AlignedData;
  }, [series]);

  useEffect(() => {
    if (!chartRef.current || plotRef.current || !hasData) {
      return;
    }
    const element = chartRef.current;
    const width = element.clientWidth || 640;
    const axisFormat =
      granularity === "1d"
        ? "{MM}-{DD}"
        : granularity === "1h"
          ? "{MM}-{DD} {HH}:00"
          : "{HH}:{mm}";
    const tooltipFormat =
      granularity === "1d" ? "{YYYY}-{MM}-{DD}" : "{YYYY}-{MM}-{DD} {HH}:{mm}";
    const axisFormatter = uPlot.fmtDate(axisFormat);
    const tooltipFormatter = uPlot.fmtDate(tooltipFormat);
    const spline = uPlot.paths?.spline ? uPlot.paths.spline() : undefined;
    const opts: uPlot.Options = {
      width,
      height: 180,
      scales: {
        x: { time: true },
        typing: {},
        keys: {},
      },
      axes: [
        {
          scale: "x",
          values: (_, ticks) =>
            ticks.map((tick) => axisFormatter(new Date(tick * 1000))),
          grid: { show: false },
        },
        {
          scale: "typing",
          label: "打字时间(秒)",
          grid: { show: false },
        },
        {
          scale: "keys",
          label: "按键次数",
          side: 1,
          grid: { show: false },
        },
      ],
      series: [
        {
          label: "时间",
          value: (_, v) =>
            v == null ? "" : tooltipFormatter(new Date(v * 1000)),
        },
        {
          label: "打字时间",
          scale: "typing",
          stroke: "#0f172a",
          width: 2,
          paths: spline,
          points: { show: false },
        },
        {
          label: "按键次数",
          scale: "keys",
          stroke: "#3b82f6",
          width: 2,
          paths: spline,
          points: { show: false },
        },
      ],
    };
    const plot = new uPlot(opts, data, element);
    plotRef.current = plot;
    return () => {
      plot.destroy();
      plotRef.current = null;
    };
  }, [data, hasData, granularity, series.timestamps]);

  useEffect(() => {
    if (plotRef.current && hasData) {
      plotRef.current.setData(data);
    }
  }, [data, hasData]);

  useEffect(() => {
    if (!chartRef.current) {
      return;
    }
    const element = chartRef.current;
    const resize = () => {
      if (!plotRef.current) {
        return;
      }
      const width = element.clientWidth || 640;
      plotRef.current.setSize({ width, height: 180 });
    };
    resize();
    let observer: ResizeObserver | null = null;
    if (typeof ResizeObserver !== "undefined") {
      observer = new ResizeObserver(resize);
      observer.observe(element);
    } else {
      window.addEventListener("resize", resize);
    }
    return () => {
      if (observer) {
        observer.disconnect();
      } else {
        window.removeEventListener("resize", resize);
      }
    };
  }, []);

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
        {!hasData ? (
          <div className="table-empty">暂无数据</div>
        ) : (
          <div ref={chartRef} />
        )}
      </div>
    </section>
  );
}

export default TrendChart;
