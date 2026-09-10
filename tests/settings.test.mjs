import test from 'node:test';
import assert from 'node:assert/strict';
import { defaultSettings, numberFields, validateSettings } from '../src/settings.ts';

test('settings validate all integer boundaries and empty input', () => {
  assert.equal(validateSettings(defaultSettings), null);
  for (const [key, { min, max }] of Object.entries(numberFields)) {
    for (const value of [min, max]) assert.equal(validateSettings({ ...defaultSettings, [key]: value }), null);
    for (const value of ['', min - 1, max + 1, min + .5, NaN, Infinity]) assert.ok(validateSettings({ ...defaultSettings, [key]: value }));
  }
});
