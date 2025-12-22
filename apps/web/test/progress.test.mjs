import test from 'node:test';
import assert from 'node:assert/strict';

import { createHistory, historyPush, historyRedo, historyUndo, makePuzzleKey } from '../src/lib/progress.js';

test('makePuzzleKey: daily/seed/custom', () => {
	assert.equal(makePuzzleKey('daily', { dateYmd: '2025-12-21' }), 'daily:2025-12-21');
	assert.equal(makePuzzleKey('seed', { seed: '123' }), 'seed:123');
	assert.equal(makePuzzleKey('custom', { levelCode: 'abc' }), 'level:abc');
});

test('history: push/undo/redo basic', () => {
	let h = createHistory(0n);
	h = historyPush(h, 1n);
	h = historyPush(h, 3n);
	assert.equal(h.present, 3n);
	assert.deepEqual(h.undo, [0n, 1n]);
	assert.deepEqual(h.redo, []);

	h = historyUndo(h);
	assert.equal(h.present, 1n);
	assert.deepEqual(h.redo, [3n]);

	h = historyRedo(h);
	assert.equal(h.present, 3n);
	assert.deepEqual(h.undo, [0n, 1n]);
});
