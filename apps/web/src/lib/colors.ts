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
			return 'var(--c-red)';
		case Color.Blue:
			return 'var(--c-blue)';
		case Color.Black:
			return 'var(--c-black)';
		case Color.Green:
			return 'var(--c-green)';
		case Color.Yellow:
			return 'var(--c-yellow)';
		case Color.Purple:
			return 'var(--c-purple)';
		case Color.White:
			return 'var(--c-white)';
		case Color.Orange:
			return 'var(--c-orange)';
		case Color.Cyan:
			return 'var(--c-cyan)';
		default:
			return 'var(--c-unknown)';
	}
}
