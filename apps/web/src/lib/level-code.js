// 关卡编码/解码（用于 URL 分享与编辑器导入导出）
//
// 设计目标：
// - 可逆、短链接、无需额外依赖；
// - 支持版本号，便于未来升级格式；
// - Node 与浏览器均可运行（Node 测试用 Buffer，浏览器用 btoa/atob）。

export const LEVEL_VERSION = 1;
export const GRID_SIZE = 5;
export const CELL_COUNT = GRID_SIZE * GRID_SIZE;

/**
 * 将 25 个 0..15 的值打包为 nibble（低 4bit / 高 4bit），并在首字节写入版本号。
 * @param {number[]} grid
 * @returns {Uint8Array}
 */
function packLevelBytes(grid) {
	if (!Array.isArray(grid) || grid.length !== CELL_COUNT) {
		throw new Error(`grid 必须是长度为 ${CELL_COUNT} 的数组`);
	}

	const bytes = new Uint8Array(1 + Math.ceil(CELL_COUNT / 2));
	bytes[0] = LEVEL_VERSION;

	for (let i = 0; i < CELL_COUNT; i++) {
		const v = grid[i];
		if (!Number.isInteger(v) || v < 0 || v > 15) {
			throw new Error(`grid[${i}] 非法：${String(v)}（要求 0..15 整数）`);
		}
		const bi = 1 + (i >> 1);
		if ((i & 1) === 0) bytes[bi] = v & 0xf;
		else bytes[bi] |= (v & 0xf) << 4;
	}

	return bytes;
}

/**
 * @param {Uint8Array} bytes
 * @returns {{ version: number, grid: number[] }}
 */
function unpackLevelBytes(bytes) {
	if (!(bytes instanceof Uint8Array) || bytes.length < 2) {
		throw new Error('bytes 非法');
	}

	const version = bytes[0];
	if (version !== LEVEL_VERSION) {
		throw new Error(`不支持的关卡版本：${version}`);
	}

	const grid = new Array(CELL_COUNT).fill(0);
	for (let i = 0; i < CELL_COUNT; i++) {
		const bi = 1 + (i >> 1);
		const b = bytes[bi] ?? 0;
		grid[i] = (i & 1) === 0 ? b & 0xf : (b >> 4) & 0xf;
	}

	return { version, grid };
}

/**
 * @param {Uint8Array} bytes
 * @returns {string}
 */
function bytesToBase64Url(bytes) {
	// Node
	if (typeof Buffer !== 'undefined') {
		return Buffer.from(bytes)
			.toString('base64')
			.replaceAll('+', '-')
			.replaceAll('/', '_')
			.replaceAll('=', '');
	}

	// 浏览器
	let binary = '';
	for (const b of bytes) binary += String.fromCharCode(b);
	return btoa(binary).replaceAll('+', '-').replaceAll('/', '_').replaceAll('=', '');
}

/**
 * @param {string} s
 * @returns {Uint8Array}
 */
function base64UrlToBytes(s) {
	if (typeof s !== 'string' || !s.trim()) throw new Error('level 不能为空');
	let b64 = s.replaceAll('-', '+').replaceAll('_', '/');
	const pad = b64.length % 4;
	if (pad === 2) b64 += '==';
	else if (pad === 3) b64 += '=';
	else if (pad !== 0) throw new Error('level base64 长度非法');

	// Node
	if (typeof Buffer !== 'undefined') {
		return new Uint8Array(Buffer.from(b64, 'base64'));
	}

	// 浏览器
	const bin = atob(b64);
	const out = new Uint8Array(bin.length);
	for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
	return out;
}

/**
 * 编码关卡为短字符串（适合放在 `?level=`）。
 * @param {number[]} gridFlat25
 * @returns {string}
 */
export function encodeLevel(gridFlat25) {
	return bytesToBase64Url(packLevelBytes(gridFlat25));
}

/**
 * 解码 `?level=` 得到关卡。
 * @param {string} code
 * @returns {{ version: number, grid: number[] }}
 */
export function decodeLevel(code) {
	return unpackLevelBytes(base64UrlToBytes(code));
}

/**
 * 生成“关卡 JSON”导出对象：`{ version: 1, grid: number[25] }`
 * @param {number[]} gridFlat25
 */
export function levelToJson(gridFlat25) {
	return { version: LEVEL_VERSION, grid: [...gridFlat25] };
}

/**
 * 将导入 JSON（支持 `number[25]` 或 `number[5][5]`）规范化为 `number[25]`。
 * @param {unknown} value
 * @returns {number[]}
 */
export function normalizeLevelJson(value) {
	if (!value || typeof value !== 'object') throw new Error('JSON 必须是对象');

	/** @type {{ version?: unknown, grid?: unknown }} */
	// @ts-ignore
	const obj = value;
	if (obj.version !== LEVEL_VERSION) throw new Error(`version 必须为 ${LEVEL_VERSION}`);

	const grid = obj.grid;
	if (!grid) throw new Error('缺少 grid 字段');

	// flat
	if (Array.isArray(grid) && grid.length === CELL_COUNT && grid.every((x) => Number.isInteger(x))) {
		return /** @type {number[]} */ (grid);
	}

	// 5x5
	if (Array.isArray(grid) && grid.length === GRID_SIZE) {
		const flat = [];
		for (let r = 0; r < GRID_SIZE; r++) {
			const row = grid[r];
			if (!Array.isArray(row) || row.length !== GRID_SIZE) throw new Error('grid 必须是 5x5');
			for (let c = 0; c < GRID_SIZE; c++) {
				const v = row[c];
				if (!Number.isInteger(v)) throw new Error('grid 必须全部为整数');
				flat.push(v);
			}
		}
		return flat;
	}

	throw new Error('grid 必须是 number[25] 或 number[5][5]');
}

