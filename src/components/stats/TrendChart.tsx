import { useEffect, useMemo, useRef } from "react";
import { Accordion, Box, Button, ButtonGroup, Flex, HStack, Text } from "@chakra-ui/react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { TrendGranularity, TrendSeries } from "../../types";
import { glassPillStyle, glassSubtleStyle, glassSurfaceStyle } from "../../styles/glass";

type TrendChartProps = {
  series: TrendSeries;
  granularity: TrendGranularity;
  onGranularityChange: (value: TrendGranularity) => void;
};

function TrendChart({ series, granularity, onGranularityChange }: TrendChartProps) {
  const chartRef = useRef<HTMLDivElement | null>(null);
  const averageChartRef = useRef<HTMLDivElement | null>(null);
  const plotRef = useRef<uPlot | null>(null);
  const averagePlotRef = useRef<uPlot | null>(null);
  const hasData = series.timestamps.length > 0;

  const data = useMemo<uPlot.AlignedData>(
    () => [series.timestamps, series.activeSeconds, series.keyCounts] as uPlot.AlignedData,
    [series],
  );
  const averageData = useMemo<uPlot.AlignedData>(
    () =>
      [
        series.timestamps,
        series.averageActiveSecondsPerSession,
        series.averageKeysPerSession,
      ] as uPlot.AlignedData,
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
    if (!averageChartRef.current || averagePlotRef.current || !hasData) return;

    const element = averageChartRef.current;
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
      scales: { x: { time: true }, avgTyping: {}, avgKeys: {} },
      axes: [
        {
          scale: "x",
          values: (_, ticks) => axisValues(ticks),
          space: granularity === "1h" ? 70 : 40,
          grid: { show: false },
        },
        { scale: "avgTyping", label: "平均每段时长(秒)", grid: { show: false } },
        { scale: "avgKeys", label: "平均每段按键", side: 1, grid: { show: false } },
      ],
      series: [
        { label: "时间", value: (_, v) => (v == null ? "" : tooltipFormatter(new Date(v * 1000))) },
        { label: "平均每段时长", scale: "avgTyping", stroke: "#0ea5e9", width: 2, paths: spline, points: { show: false } },
        { label: "平均每段按键", scale: "avgKeys", stroke: "#22c55e", width: 2, paths: spline, points: { show: false } },
      ],
    };

    const plot = new uPlot(opts, averageData, element);
    averagePlotRef.current = plot;

    return () => {
      plot.destroy();
      averagePlotRef.current = null;
    };
  }, [averageData, hasData, granularity]);

  useEffect(() => {
    if (averagePlotRef.current && hasData) averagePlotRef.current.setData(averageData);
  }, [averageData, hasData]);

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

  useEffect(() => {
    if (!averageChartRef.current) return;
    const element = averageChartRef.current;
    const resize = () => {
      if (!averagePlotRef.current) return;
      const width = element.clientWidth || 640;
      averagePlotRef.current.setSize({ width, height: 220 });
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
    <Box {...glassSurfaceStyle} borderRadius="16px" p="6">
      <Flex justify="space-between" align="center" gap="3" flexWrap="wrap" mb="4">
        <Text fontSize="xl" fontWeight="semibold">键盘活跃度</Text>
        <ButtonGroup size="sm" gap="1" {...glassPillStyle} borderRadius="999px" p="1">
          <Button
            onClick={() => onGranularityChange("1m")}
            variant="ghost"
            borderRadius="999px"
            bg={granularity === "1m" ? "rgba(255,255,255,0.82)" : "transparent"}
            boxShadow={granularity === "1m" ? "sm" : "none"}
          >
            1分钟
          </Button>
          <Button
            onClick={() => onGranularityChange("5m")}
            variant="ghost"
            borderRadius="999px"
            bg={granularity === "5m" ? "rgba(255,255,255,0.82)" : "transparent"}
            boxShadow={granularity === "5m" ? "sm" : "none"}
          >
            5分钟
          </Button>
          <Button
            onClick={() => onGranularityChange("1h")}
            variant="ghost"
            borderRadius="999px"
            bg={granularity === "1h" ? "rgba(255,255,255,0.82)" : "transparent"}
            boxShadow={granularity === "1h" ? "sm" : "none"}
          >
            1小时
          </Button>
          <Button
            onClick={() => onGranularityChange("1d")}
            variant="ghost"
            borderRadius="999px"
            bg={granularity === "1d" ? "rgba(255,255,255,0.82)" : "transparent"}
            boxShadow={granularity === "1d" ? "sm" : "none"}
          >
            1天
          </Button>
        </ButtonGroup>
      </Flex>
      {hasData ? (
        <>
          <Box
            ref={chartRef}
            minH="220px"
            borderRadius="12px"
            bg="rgba(255,255,255,0.32)"
            borderWidth="1px"
            borderColor="glass.borderSoft"
            p="1"
          />
          <Accordion.Root mt="5" collapsible defaultValue={[]}>
            <Accordion.Item value="average-trend" {...glassSubtleStyle} borderRadius="12px">
              <Accordion.ItemTrigger px="4" py="3">
                <HStack justify="space-between" w="full">
                  <Text fontSize="sm" fontWeight="semibold" color="gray.700">输入强度趋势</Text>
                  <Accordion.ItemIndicator />
                </HStack>
              </Accordion.ItemTrigger>
              <Accordion.ItemContent>
                <Accordion.ItemBody px="0" pb="1">
                  <Box
                    ref={averageChartRef}
                    minH="220px"
                    borderRadius="12px"
                    bg="rgba(255,255,255,0.28)"
                    borderWidth="1px"
                    borderColor="glass.borderSoft"
                    p="1"
                  />
                </Accordion.ItemBody>
              </Accordion.ItemContent>
            </Accordion.Item>
          </Accordion.Root>
        </>
      ) : (
        <Text color="gray.500" textAlign="center" py="10">暂无数据</Text>
      )}
    </Box>
  );
}

export default TrendChart;
