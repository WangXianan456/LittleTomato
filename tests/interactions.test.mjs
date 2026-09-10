import test from 'node:test';
import assert from 'node:assert/strict';
import { petMood, progressPercent, interactionMessage } from '../src/interactions.ts';
import { characters } from '../src/characters.ts';

test('pet mood follows phase and status without treating paused breaks as active', () => {
  assert.equal(petMood(null), 'idle');
  for (const phase of ['focus', 'short_break', 'long_break']) {
    for (const status of ['ready', 'paused']) assert.equal(petMood({ phase, status }), 'idle');
    assert.equal(petMood({ phase, status: 'completed' }), 'celebrate');
    assert.equal(petMood({ phase, status: 'running' }), phase === 'focus' ? 'focus' : 'rest');
  }
});
test('progress uses current phase duration and stays within bounds', () => {
  assert.equal(progressPercent(null), 0);
  for (const [plannedSeconds, remainingSeconds, expected] of [[1500,750,50],[300,0,100],[0,0,0],[300,400,0],[300,-1,100]]) {
    assert.equal(progressPercent({ plannedSeconds, remainingSeconds }), expected);
  }
});
test('every available character has short responses to all interactions', () => {
  for (const character of characters) for (const action of ['pet', 'wave', 'stretch']) {
    const message = interactionMessage(character.id, action);
    assert.ok(message.length > 0 && message.length <= 32);
  }
});
