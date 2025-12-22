import test from 'node:test';
import assert from 'node:assert/strict';

import { normalizeDismissedAt, shouldAutoShowTutorial } from '../src/lib/tutorial.js';

test('tutorial: normalizeDismissedAt', () => {
	assert.equal(normalizeDismissedAt(null), null);
	assert.equal(normalizeDismissedAt(''), null);
	assert.equal(normalizeDismissedAt('  '), null);
	assert.equal(normalizeDismissedAt('not-iso'), null);
	assert.equal(normalizeDismissedAt('2025-12-22T00:00:00.000Z'), '2025-12-22T00:00:00.000Z');
});

test('tutorial: shouldAutoShowTutorial', () => {
	assert.equal(shouldAutoShowTutorial(null), true);
	assert.equal(shouldAutoShowTutorial(''), true);
	assert.equal(shouldAutoShowTutorial('2025-12-22T00:00:00.000Z'), false);
});

