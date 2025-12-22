export const MARK_NONE: 0;
export const MARK_EXCLUDE: 1;
export const MARK_QUESTION: 2;

export function createMarks(length?: number): number[];
export function normalizeMarks(raw: any, length?: number): number[];
export function cycleMarkValue(v: number): number;

