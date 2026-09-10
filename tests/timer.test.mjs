import test from "node:test";
import assert from "node:assert/strict";
import { formatTime, primaryAction, recoveryMessage } from "../src/timer.ts";

test("remaining time displays minute boundaries and never goes negative", () => {
  assert.equal(formatTime(1500), "25:00");
  assert.equal(formatTime(59.2), "01:00");
  assert.equal(formatTime(-1), "00:00");
});
test("completed phases require an explicit next action", () => {
  assert.deepEqual(primaryAction({ status: "completed", phase: "focus" }), { action: "next", label: "休息" });
  assert.deepEqual(primaryAction({ status: "completed", phase: "long_break" }), { action: "next", label: "专注" });
  assert.equal(primaryAction({ status: "paused" }).action, "resume");
});

test("recovery explains system interruption and supports old snapshots", () => {
  assert.match(recoveryMessage("locked"), /锁屏/);
  assert.match(recoveryMessage("sleep"), /休眠/);
  assert.match(recoveryMessage(undefined), /上次/);
});
