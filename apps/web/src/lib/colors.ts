export const Color = {
	Red: 0,
	Blue: 1,
	Black: 2,
	Green: 3,
	Yellow: 4,
	Purple: 5,
	White: 6,
	Orange: 7,
	Cyan: 8
} as const;

export type ColorId = (typeof Color)[keyof typeof Color];

export function colorToCss(color: ColorId): string {
	switch (color) {
		case Color.Red:
			return '#ef4444';
		case Color.Blue:
			return '#3b82f6';
		case Color.Black:
			return '#0f172a';
		case Color.Green:
			return '#22c55e';
		case Color.Yellow:
			return '#eab308';
		case Color.Purple:
			return '#a855f7';
		case Color.White:
			return '#f8fafc';
		case Color.Orange:
			return '#f97316';
		case Color.Cyan:
			return '#06b6d4';
		default:
			return '#94a3b8';
	}
}

