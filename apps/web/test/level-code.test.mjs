import test from 'node:test';
import assert from 'node:assert/strict';

import { decodeLevel, encodeLevel, levelToJson, normalizeLevelJson } from '../src/lib/level-code.js';

test('encodeLevel/decodeLevel: roundtrip', () => {
	const grid = Array.from({ length: 25 }, (_, i) => i % 9);
	const code = encodeLevel(grid);
	const decoded = decodeLevel(code);
	assert.equal(decoded.version, 1);
	assert.deepEqual(decoded.grid, grid);
});

test('normalizeLevelJson: accepts {version, grid:number[25]}', () => {
	const grid = Array.from({ length: 25 }, (_, i) => (i * 3) % 9);
	const obj = levelToJson(grid);
	assert.deepEqual(normalizeLevelJson(obj), grid);
});

test('normalizeLevelJson: accepts {version, grid:number[5][5]}', () => {
	const flat = Array.from({ length: 25 }, (_, i) => (i + 2) % 9);
	const grid2d = [];
	for (let r = 0; r < 5; r++) grid2d.push(flat.slice(r * 5, r * 5 + 5));

	assert.deepEqual(normalizeLevelJson({ version: 1, grid: grid2d }), flat);
});

