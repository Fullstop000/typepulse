import { useEffect, useMemo, useRef } from "react";
import { Box, Button, ButtonGroup, Flex, Text } from "@chakra-ui/react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { TrendGranularity, TrendSeries } from "../types";

type TrendChartProps = {
  series: TrendSeries;
  granularity: TrendGranularity;
  onGranularityChange: (value: TrendGranularity) => void;
};

function TrendChart({ series, granularity, onGranularityChange }: TrendChartProps) {
  const chartRef = useRef<HTMLDivElement | null>(null);
  const plotRef = useRef<uPlot | null>(null);
  const hasData = series.timestamps.length > 0;

  const data = useMemo<uPlot.AlignedData>(
    () => [series.timestamps, series.activeSeconds, series.keyCounts] as uPlot.AlignedData,
    [series],
  );

  useEffect(() => {
    if (!chartRef.current || plotRef.current || !hasData) return;

    const element = chartRef.current;
    const width = element.clientWidth || 640;
    const axisFormat = granularity === "1d" ? "{MM}-{DD}" : "{HH}:{mm}";
    const tooltipFormat = granularity === "1d" ? "{YYYY}-{MM}-{DD}" : "{YYYY}-{MM}-{DD} {HH}:{mm}";
    const axisFormatter = uPlot.fmtDate(axisFormat);
    const tooltipFormatter = uPlot.fmtDate(tooltipFormat);
    const hourFormatter = uPlot.fmtDate("{HH}:00");
    const dayFormatter = uPlot.fmtDate("{MM}-{DD}");

    const axisValues =
      granularity === "1h"
        ? (ticks: number[]) => {
            let lastDay = "";
            return ticks.map((tick) => {
              const date = new Date(tick * 1000);
              const day = dayFormatter(date);
              const hour = hourFormatter(date);
              if (day !== lastDay) {
                lastDay = day;
                return `${day}\n${hour}`;
              }
              return hour;
            });
          }
        : (ticks: number[]) => ticks.map((tick) => axisFormatter(new Date(tick * 1000)));

    const spline = uPlot.paths?.spline ? uPlot.paths.spline() : undefined;
    const opts: uPlot.Options = {
      width,
      height: 220,
      scales: { x: { time: true }, typing: {}, keys: {} },
      axes: [
        {
          scale: "x",
          values: (_, ticks) => axisValues(ticks),
          space: granularity === "1h" ? 70 : 40,
          grid: { show: false },
        },
        { scale: "typing", label: "打字时间(秒)", grid: { show: false } },
        { scale: "keys", label: "按键次数", side: 1, grid: { show: false } },
      ],
      series: [
        { label: "时间", value: (_, v) => (v == null ? "" : tooltipFormatter(new Date(v * 1000))) },
        { label: "打字时间", scale: "typing", stroke: "#0f172a", width: 2, paths: spline, points: { show: false } },
        { label: "按键次数", scale: "keys", stroke: "#3b82f6", width: 2, paths: spline, points: { show: false } },
      ],
    };

    const plot = new uPlot(opts, data, element);
    plotRef.current = plot;

    return () => {
      plot.destroy();
      plotRef.current = null;
    };
  }, [data, hasData, granularity]);

  useEffect(() => {
    if (plotRef.current && hasData) plotRef.current.setData(data);
  }, [data, hasData]);

  useEffect(() => {
    if (!chartRef.current) return;
    const element = chartRef.current;
    const resize = () => {
      if (!plotRef.current) return;
      const width = element.clientWidth || 640;
      plotRef.current.setSize({ width, height: 220 });
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
      if (observer) observer.disconnect();
      else window.removeEventListener("resize", resize);
    };
  }, []);

  return (
    <Box bg="white" borderRadius="16px" p="6" boxShadow="sm">
      <Flex justify="space-between" align="center" gap="3" flexWrap="wrap" mb="4">
        <Text fontSize="xl" fontWeight="semibold">键盘活跃度</Text>
        <ButtonGroup size="sm" variant="outline" attached>
          <Button onClick={() => onGranularityChange("1m")} variant={granularity === "1m" ? "solid" : "outline"}>1分钟</Button>
          <Button onClick={() => onGranularityChange("5m")} variant={granularity === "5m" ? "solid" : "outline"}>5分钟</Button>
          <Button onClick={() => onGranularityChange("1h")} variant={granularity === "1h" ? "solid" : "outline"}>1小时</Button>
          <Button onClick={() => onGranularityChange("1d")} variant={granularity === "1d" ? "solid" : "outline"}>1天</Button>
        </ButtonGroup>
      </Flex>
      {hasData ? (
        <Box ref={chartRef} minH="220px" />
      ) : (
        <Text color="gray.500" textAlign="center" py="10">暂无数据</Text>
      )}
    </Box>
  );
}

export default TrendChart;
