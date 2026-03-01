import { FilterRange, StatsRow, TrendGranularity, TrendSeries } from "../types";

export const parseRowDate = (value: string) =>
  value.includes(" ")
    ? new Date(value.replace(" ", "T"))
    : new Date(`${value}T00:00:00`);

const minuteKeyFromDate = (date: Date) => {
  const yyyy = date.getFullYear();
  const mm = String(date.getMonth() + 1).padStart(2, "0");
  const dd = String(date.getDate()).padStart(2, "0");
  const hh = String(date.getHours()).padStart(2, "0");
  const min = String(date.getMinutes()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd} ${hh}:${min}`;
};

const floorToBucket = (date: Date, bucketMinutes: number) => {
  const floored = new Date(date);
  if (bucketMinutes >= 1440) {
    floored.setHours(0, 0, 0, 0);
    return floored;
  }
  if (bucketMinutes >= 60) {
    floored.setMinutes(0, 0, 0);
    return floored;
  }
  const minutes = floored.getMinutes();
  const bucketStart = Math.floor(minutes / bucketMinutes) * bucketMinutes;
  floored.setMinutes(bucketStart, 0, 0);
  return floored;
};

export const formatMs = (ms: number) => {
  const totalSeconds = Math.floor(ms / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor(totalSeconds / 60);
  const minuteRemainder = minutes % 60;
  const seconds = totalSeconds % 60;
  if (hours > 0) {
    return `${hours}h ${minuteRemainder}m ${seconds}s`;
  }
  return `${minutes}m ${seconds}s`;
};

export const buildTrendSeries = (
  rows: StatsRow[],
  granularity: TrendGranularity,
  filterRange: FilterRange,
): TrendSeries => {
  // Time window follows the overview filter; granularity only controls bucket size.
  const bucketMinutes = {
    "1m": 1,
    "5m": 5,
    "1h": 60,
    "1d": 24 * 60,
  }[granularity];
  const bucketMs = bucketMinutes * 60 * 1000;
  const now = new Date();
  const todayStart = new Date(now);
  todayStart.setHours(0, 0, 0, 0);
  const yesterdayStart = new Date(todayStart);
  yesterdayStart.setDate(yesterdayStart.getDate() - 1);
  const sevenDaysStart = new Date(todayStart);
  sevenDaysStart.setDate(sevenDaysStart.getDate() - 6);
  const endOfYesterday = new Date(todayStart.getTime() - bucketMs);
  const end = (() => {
    if (filterRange === "today") {
      return granularity === "1d" ? todayStart : floorToBucket(now, bucketMinutes);
    }
    if (filterRange === "yesterday") {
      return granularity === "1d" ? yesterdayStart : endOfYesterday;
    }
    return granularity === "1d" ? todayStart : floorToBucket(now, bucketMinutes);
  })();
  const start = (() => {
    if (filterRange === "today") {
      return todayStart;
    }
    if (filterRange === "yesterday") {
      return yesterdayStart;
    }
    return sevenDaysStart;
  })();
  const bucketCount = Math.max(
    1,
    Math.floor((end.getTime() - start.getTime()) / bucketMs) + 1,
  );
  // Aggregate raw rows into the selected time bucket for trend computations.
  const buckets = new Map<string, { activeMs: number; keyCount: number; sessionCount: number }>();
  for (const row of rows) {
    const rowDate = parseRowDate(row.date);
    const bucketDate = floorToBucket(rowDate, bucketMinutes);
    // Validate by bucket range so current in-progress bucket (especially 1d granularity) is included.
    if (bucketDate < start || bucketDate > end) {
      continue;
    }
    const key = minuteKeyFromDate(bucketDate);
    const current = buckets.get(key) ?? { activeMs: 0, keyCount: 0, sessionCount: 0 };
    current.activeMs += row.active_typing_ms;
    current.keyCount += row.key_count;
    current.sessionCount += row.session_count;
    buckets.set(key, current);
  }
  const timestamps: number[] = [];
  const activeSeconds: number[] = [];
  const keyCounts: number[] = [];
  const averageActiveSecondsPerSession: number[] = [];
  const averageKeysPerSession: number[] = [];
  for (let i = 0; i < bucketCount; i += 1) {
    const pointDate = new Date(
      start.getTime() + i * bucketMs,
    );
    const key = minuteKeyFromDate(pointDate);
    const value = buckets.get(key) ?? { activeMs: 0, keyCount: 0, sessionCount: 0 };
    const averageActiveSeconds =
      value.sessionCount > 0 ? Math.round(value.activeMs / 1000 / value.sessionCount) : 0;
    const averageKeys = value.sessionCount > 0 ? value.keyCount / value.sessionCount : 0;
    timestamps.push(Math.floor(pointDate.getTime() / 1000));
    activeSeconds.push(Math.round(value.activeMs / 1000));
    keyCounts.push(value.keyCount);
    averageActiveSecondsPerSession.push(averageActiveSeconds);
    averageKeysPerSession.push(Number(averageKeys.toFixed(2)));
  }
  return {
    timestamps,
    activeSeconds,
    keyCounts,
    averageActiveSecondsPerSession,
    averageKeysPerSession,
  };
};
