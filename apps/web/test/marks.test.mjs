import test from 'node:test';
import assert from 'node:assert/strict';

import { createMarks, cycleMarkValue, normalizeMarks, MARK_EXCLUDE, MARK_NONE, MARK_QUESTION } from '../src/lib/marks.js';

test('marks: createMarks length + defaults', () => {
	const m = createMarks();
	assert.equal(m.length, 25);
	assert.ok(m.every((v) => v === MARK_NONE));
});

test('marks: cycleMarkValue cycles none -> exclude -> question -> none', () => {
	assert.equal(cycleMarkValue(MARK_NONE), MARK_EXCLUDE);
	assert.equal(cycleMarkValue(MARK_EXCLUDE), MARK_QUESTION);
	assert.equal(cycleMarkValue(MARK_QUESTION), MARK_NONE);
	assert.equal(cycleMarkValue(999), MARK_NONE);
});

test('marks: normalizeMarks clamps shape + values', () => {
	const raw = Array.from({ length: 30 }, (_, i) => (i % 3));
	raw[1] = 999;
	const m = normalizeMarks(raw);
	assert.equal(m.length, 25);
	assert.equal(m[0], MARK_NONE);
	assert.equal(m[1], MARK_NONE);
	assert.equal(m[2], MARK_QUESTION);
});

