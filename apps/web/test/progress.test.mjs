import test from 'node:test';
import assert from 'node:assert/strict';

import { createHistory, historyPush, historyRedo, historyUndo, makePuzzleKey } from '../src/lib/progress.js';

test('makePuzzleKey: daily/seed/custom', () => {
	assert.equal(makePuzzleKey('daily', { dateYmd: '2025-12-21' }), 'daily:2025-12-21');
	assert.equal(makePuzzleKey('seed', { seed: '123' }), 'seed:123');
	assert.equal(makePuzzleKey('custom', { levelCode: 'abc' }), 'level:abc');
});

test('history: push/undo/redo basic', () => {
	let h = createHistory(0);
	h = historyPush(h, 1);
	h = historyPush(h, 3);
	assert.equal(h.present, 3);
	assert.deepEqual(h.undo, [0, 1]);
	assert.deepEqual(h.redo, []);

	h = historyUndo(h);
	assert.equal(h.present, 1);
	assert.deepEqual(h.redo, [3]);

	h = historyRedo(h);
	assert.equal(h.present, 3);
	assert.deepEqual(h.undo, [0, 1]);
});

