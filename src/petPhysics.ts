export function gazeTarget(x: number, y: number, centerX: number, centerY: number) {
  const dx = x - centerX, dy = y - centerY;
  const distance = Math.hypot(dx, dy);
  const strength = Math.min(1, distance / 240);
  return distance < 1 ? { x: 0, y: 0 } : { x: dx / distance * strength, y: dy / distance * strength };
}
export function springStep(position: number, velocity: number, target: number, dt: number) {
  const step = Math.min(Math.max(dt, 0), 1 / 30);
  const nextVelocity = velocity + ((target - position) * 150 - velocity * 20) * step;
  return { position: position + nextVelocity * step, velocity: nextVelocity };
}
