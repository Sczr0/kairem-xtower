export const TUTORIAL_VERSION: number;
export const TUTORIAL_DISMISSED_AT_KEY: string;

export function normalizeDismissedAt(raw: any): string | null;
export function shouldAutoShowTutorial(dismissedAtIso: string | null): boolean;
export function readTutorialDismissedAt(): string | null;
export function setTutorialDismissed(dismissed: boolean): void;

