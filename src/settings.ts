import type { CharacterId } from "./characters";
export type Settings = {
  character: CharacterId;
  focusMinutes: number;
  shortBreakMinutes: number;
  longBreakMinutes: number;
  roundsBeforeLongBreak: number;
  petSize: number;
  alwaysOnTop: boolean;
  reducedMotion: boolean;
  theme: "system" | "light" | "dark";
  launchOnStartup: boolean;
};
export const defaultSettings: Settings = {
  character: "tomato",
  focusMinutes: 25, shortBreakMinutes: 5, longBreakMinutes: 15,
  roundsBeforeLongBreak: 4, petSize: 160, alwaysOnTop: true,
  reducedMotion: false, theme: "system", launchOnStartup: false,
};
export const numberFields = {
  focusMinutes: { label: "专注时长", min: 1, max: 180, unit: "分钟" },
  shortBreakMinutes: { label: "短休息", min: 1, max: 60, unit: "分钟" },
  longBreakMinutes: { label: "长休息", min: 1, max: 120, unit: "分钟" },
  roundsBeforeLongBreak: { label: "长休息间隔", min: 2, max: 12, unit: "轮专注" },
  petSize: { label: "桌宠大小", min: 80, max: 240, unit: "像素" },
} as const;
export type NumberKey = keyof typeof numberFields;
export type SettingsDraft = Omit<Settings, NumberKey> & Record<NumberKey, number | "">;
export function validateSettings(draft: SettingsDraft): string | null {
  for (const key of Object.keys(numberFields) as NumberKey[]) {
    const value = draft[key]; const { label, min, max } = numberFields[key];
    if (typeof value !== "number" || !Number.isInteger(value) || value < min || value > max) return `${label}请填写 ${min}–${max} 之间的整数。`;
  }
  return null;
}
