import test from 'node:test';
import assert from 'node:assert/strict';
import { gazeTarget, springStep } from '../src/petPhysics.ts';
test('gaze follows desktop coordinates, including negative monitor positions, within a unit circle', () => {
  assert.deepEqual(gazeTarget(100,100,100,100), {x:0,y:0});
  assert.deepEqual(gazeTarget(-1000,100,100,100), {x:-1,y:0});
  assert.deepEqual(gazeTarget(100,1000,100,100), {x:0,y:1});
  const diagonal = gazeTarget(9000,-9000,100,100);
  assert.ok(Math.hypot(diagonal.x,diagonal.y) <= 1.000001);
});
test('spring settles after lift and release, including delayed frames', () => {
  let state = {position:0,velocity:0};
  for(let i=0;i<120;i++) state=springStep(state.position,state.velocity,1,1/60);
  assert.ok(Math.abs(state.position-1)<.001);
  state=springStep(state.position,state.velocity,0,10);
  for(let i=0;i<180;i++) state=springStep(state.position,state.velocity,0,1/60);
  assert.ok(Math.abs(state.position)<.001 && Math.abs(state.velocity)<.001);
});
