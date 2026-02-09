import { StatsRow, TrendGranularity, TrendSeries } from "../types";

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
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}m ${seconds}s`;
};

export const buildTrendSeries = (
  rows: StatsRow[],
  granularity: TrendGranularity,
): TrendSeries => {
  const config = {
    "1m": { rangeMinutes: 20, bucketMinutes: 1 },
    "5m": { rangeMinutes: 60, bucketMinutes: 5 },
    "1h": { rangeMinutes: 24 * 60, bucketMinutes: 60 },
    "1d": { rangeMinutes: 7 * 24 * 60, bucketMinutes: 24 * 60 },
  }[granularity];
  const now = new Date();
  const bucketCount = Math.ceil(config.rangeMinutes / config.bucketMinutes);
  const end = floorToBucket(now, config.bucketMinutes);
  const start = new Date(
    end.getTime() - (bucketCount - 1) * config.bucketMinutes * 60 * 1000,
  );
  const buckets = new Map<string, { activeMs: number; keyCount: number }>();
  for (const row of rows) {
    const rowDate = parseRowDate(row.date);
    if (rowDate < start || rowDate > end) {
      continue;
    }
    const bucketDate = floorToBucket(rowDate, config.bucketMinutes);
    const key = minuteKeyFromDate(bucketDate);
    const current = buckets.get(key) ?? { activeMs: 0, keyCount: 0 };
    current.activeMs += row.active_typing_ms;
    current.keyCount += row.key_count;
    buckets.set(key, current);
  }
  const timestamps: number[] = [];
  const activeSeconds: number[] = [];
  const keyCounts: number[] = [];
  for (let i = 0; i < bucketCount; i += 1) {
    const pointDate = new Date(
      start.getTime() + i * config.bucketMinutes * 60 * 1000,
    );
    const key = minuteKeyFromDate(pointDate);
    const value = buckets.get(key) ?? { activeMs: 0, keyCount: 0 };
    timestamps.push(Math.floor(pointDate.getTime() / 1000));
    activeSeconds.push(Math.round(value.activeMs / 1000));
    keyCounts.push(value.keyCount);
  }
  return { timestamps, activeSeconds, keyCounts };
};
