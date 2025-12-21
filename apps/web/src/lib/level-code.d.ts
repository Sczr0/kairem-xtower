export const LEVEL_VERSION: 1;
export const GRID_SIZE: 5;
export const CELL_COUNT: 25;

export type LevelGridFlat = number[]; // 长度 25

export function encodeLevel(gridFlat25: LevelGridFlat): string;
export function decodeLevel(code: string): { version: number; grid: LevelGridFlat };

export function levelToJson(gridFlat25: LevelGridFlat): { version: number; grid: LevelGridFlat };
export function normalizeLevelJson(value: unknown): LevelGridFlat;

