// æ ‡è®°ç³»ç»Ÿï¼ˆUI ç”¨ï¼Œä¸ä¼ å¼•æ“Žï¼‰
//
// è®¾è®¡ï¼š
// - 25 æ ¼æŒ‰ç´¢å¼•å­˜ marksï¼Œæ¯?æ ¼ 0/1/2 ä¸‰æ€ï¼šæ—  / æŽ’é™¤ / é—®å·ã€?
// - ç»“æž„ä¿æŒ?çº¯å‡½æ•°ï¼Œä¾¿äºŽæµ‹è¯•ä¸Žå¤ç”¨ã€?

export const MARK_NONE = 0;
export const MARK_EXCLUDE = 1;
export const MARK_QUESTION = 2;

/**
 * @param {number} length
 * @returns {number[]}
 */
export function createMarks(length = 25) {
	return Array.from({ length }, () => MARK_NONE);
}

/**
 * @param {any} raw
 * @param {number} length
 * @returns {number[]}
 */
export function normalizeMarks(raw, length = 25) {
	if (!Array.isArray(raw)) return createMarks(length);
	const out = createMarks(length);
	for (let i = 0; i < length; i++) {
		const v = raw[i];
		if (v === MARK_NONE || v === MARK_EXCLUDE || v === MARK_QUESTION) out[i] = v;
	}
	return out;
}

/**
 * @param {number} v
 */
export function cycleMarkValue(v) {
	if (v === MARK_NONE) return MARK_EXCLUDE;
	if (v === MARK_EXCLUDE) return MARK_QUESTION;
	return MARK_NONE;
}

