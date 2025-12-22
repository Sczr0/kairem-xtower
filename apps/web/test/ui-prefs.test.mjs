import test from 'node:test';
import assert from 'node:assert/strict';

import { readBooleanFlag, writeBooleanFlag } from '../src/lib/ui-prefs.js';

test('ui-prefs: readBooleanFlag defaults', () => {
	assert.equal(readBooleanFlag(null, 'k', true), true);
	assert.equal(readBooleanFlag(undefined, 'k', false), false);
	assert.equal(readBooleanFlag({}, 'k', true), true);
});

test('ui-prefs: read/write roundtrip', () => {
	/** @type {Map<string, string>} */
	const store = new Map();

	const storage = {
		getItem(key) {
			return store.has(key) ? store.get(key) : null;
		},
		setItem(key, value) {
			store.set(key, value);
		}
	};

	assert.equal(readBooleanFlag(storage, 'x', true), true);
	writeBooleanFlag(storage, 'x', false);
	assert.equal(readBooleanFlag(storage, 'x', true), false);
	writeBooleanFlag(storage, 'x', true);
	assert.equal(readBooleanFlag(storage, 'x', false), true);
});

test('ui-prefs: storage errors are ignored', () => {
	const storage = {
		getItem() {
			throw new Error('boom');
		},
		setItem() {
			throw new Error('boom');
		}
	};

	assert.equal(readBooleanFlag(storage, 'x', true), true);
	assert.doesNotThrow(() => writeBooleanFlag(storage, 'x', true));
});

