export const STORAGE_KEY_PROGRESS_LIST_OPEN: string;

export type StorageLike = {
	getItem?: (key: string) => string | null;
	setItem?: (key: string, value: string) => void;
};

export function readBooleanFlag(
	storage: StorageLike | null | undefined,
	key: string,
	defaultValue: boolean
): boolean;

export function writeBooleanFlag(
	storage: StorageLike | null | undefined,
	key: string,
	value: boolean
): void;

export function readProgressListOpen(storage?: StorageLike | null): boolean;
export function writeProgressListOpen(value: boolean, storage?: StorageLike | null): void;

