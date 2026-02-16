import { StatsRow } from "../types";
import { parseRowDate } from "./stats";

export type ContributionCell = {
  dateKey: string;
  dayOfWeek: number;
  weekIndex: number;
  keyCount: number;
  activeTypingMs: number;
  sessionCount: number;
  level: 0 | 1 | 2 | 3 | 4;
  isToday: boolean;
};

export type ContributionMonthLabel = {
  text: string;
  weekIndex: number;
};

export type ContributionHeatmapData = {
  cells: ContributionCell[];
  monthLabels: ContributionMonthLabel[];
  weekCount: number;
  maxKeyCount: number;
  totalKeyCount: number;
  totalActiveTypingMs: number;
  totalSessionCount: number;
  activeDays: number;
};

const DAY_MS = 24 * 60 * 60 * 1000;
type DayAggregate = {
  keyCount: number;
  activeTypingMs: number;
  sessionCount: number;
};

const dayStart = (date: Date): Date => {
  const value = new Date(date);
  value.setHours(0, 0, 0, 0);
  return value;
};

const dayKey = (date: Date): string => {
  const yyyy = date.getFullYear();
  const mm = String(date.getMonth() + 1).padStart(2, "0");
  const dd = String(date.getDate()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd}`;
};

const mondayStart = (date: Date): Date => {
  const value = dayStart(date);
  const weekday = (value.getDay() + 6) % 7;
  return new Date(value.getTime() - weekday * DAY_MS);
};

const intensityLevel = (value: number, maxValue: number): 0 | 1 | 2 | 3 | 4 => {
  if (value <= 0 || maxValue <= 0) return 0;
  const ratio = value / maxValue;
  if (ratio <= 0.2) return 1;
  if (ratio <= 0.45) return 2;
  if (ratio <= 0.7) return 3;
  return 4;
};

// Build GitHub-style contribution cells from daily aggregated usage.
export const buildContributionHeatmapData = (
  rows: StatsRow[],
  options?: { monthsBack?: number },
): ContributionHeatmapData => {
  const monthsBack = Math.max(1, options?.monthsBack ?? 12);
  const today = dayStart(new Date());
  const rangeStart = dayStart(new Date(today));
  rangeStart.setMonth(rangeStart.getMonth() - monthsBack);
  const start = mondayStart(rangeStart);
  const end = today;
  const weekCount =
    Math.floor((mondayStart(end).getTime() - start.getTime()) / (7 * DAY_MS)) + 1;

  const byDay = new Map<string, DayAggregate>();
  for (const row of rows) {
    const rowDate = dayStart(parseRowDate(row.date));
    if (rowDate < rangeStart || rowDate > end) {
      continue;
    }
    const key = dayKey(rowDate);
    const aggregate = byDay.get(key) ?? {
      keyCount: 0,
      activeTypingMs: 0,
      sessionCount: 0,
    };
    aggregate.keyCount += row.key_count;
    aggregate.activeTypingMs += row.active_typing_ms;
    aggregate.sessionCount += row.session_count;
    byDay.set(key, aggregate);
  }

  let maxKeyCount = 0;
  let totalKeyCount = 0;
  let totalActiveTypingMs = 0;
  let totalSessionCount = 0;
  let activeDays = 0;
  for (const day of byDay.values()) {
    if (day.keyCount > 0) {
      activeDays += 1;
      totalKeyCount += day.keyCount;
      totalActiveTypingMs += day.activeTypingMs;
      totalSessionCount += day.sessionCount;
      if (day.keyCount > maxKeyCount) {
        maxKeyCount = day.keyCount;
      }
    }
  }

  const cells: ContributionCell[] = [];
  const monthLabels: ContributionMonthLabel[] = [];
  const monthLabelSeen = new Set<string>();

  for (let week = 0; week < weekCount; week += 1) {
    const weekStart = new Date(start.getTime() + week * 7 * DAY_MS);
    const monthLabelKey = `${weekStart.getFullYear()}-${weekStart.getMonth()}`;
    if (!monthLabelSeen.has(monthLabelKey) && weekStart.getDate() <= 7) {
      monthLabelSeen.add(monthLabelKey);
      monthLabels.push({ text: `${weekStart.getMonth() + 1}月`, weekIndex: week });
    }

    for (let weekday = 0; weekday < 7; weekday += 1) {
      const cellDate = new Date(weekStart.getTime() + weekday * DAY_MS);
      if (cellDate < rangeStart || cellDate > end) {
        continue;
      }
      const key = dayKey(cellDate);
      const day = byDay.get(key) ?? {
        keyCount: 0,
        activeTypingMs: 0,
        sessionCount: 0,
      };
      cells.push({
        dateKey: key,
        dayOfWeek: weekday,
        weekIndex: week,
        keyCount: day.keyCount,
        activeTypingMs: day.activeTypingMs,
        sessionCount: day.sessionCount,
        level: intensityLevel(day.keyCount, maxKeyCount),
        isToday: key === dayKey(today),
      });
    }
  }

  return {
    cells,
    monthLabels,
    weekCount,
    maxKeyCount,
    totalKeyCount,
    totalActiveTypingMs,
    totalSessionCount,
    activeDays,
  };
};
