import type { StockRig } from "./stockRigs";

const BOX_W = 176;
const BOX_H = 190;

/** Paper-doll rendering of an official-art cutout: the body plus articulated
 * arm/feet/ear layers reuse the same rig animations as the drawn characters,
 * and an eye overlay restores gaze, blinking and happy-eyes reactions. */
export default function StockCharacter({ rig }: { rig: StockRig }) {
  const k = Math.min(BOX_W / rig.width, BOX_H / rig.height);
  return <span className="reference-character stock-rig" aria-hidden="true" style={{ "--gaze-k": 1.12 * k } as React.CSSProperties}>
    <span className="stock-rig-inner" style={{ width: rig.width, height: rig.height, left: (BOX_W - rig.width * k) / 2, top: (BOX_H - rig.height * k) / 2, transform: `scale(${k})` }}>
      {rig.layers.map(layer => (
        <img key={layer.name} src={layer.src} alt="" draggable={false} className={layer.cls}
          style={{ left: layer.x, top: layer.y, width: layer.w, height: layer.h, zIndex: layer.z,
                   transformOrigin: layer.origin ? `${layer.origin[0] - layer.x}px ${layer.origin[1] - layer.y}px` : undefined }} />
      ))}
      {rig.eyes && <svg className="stock-eyes" width={rig.width} height={rig.height} viewBox={`0 0 ${rig.width} ${rig.height}`} fill="none">
        {rig.eyes.boxes.map((eye, i) => <g key={i} className="rig-eye">
          <g className="rig-pupil">
            <ellipse cx={eye.cx} cy={eye.cy} rx={eye.rx} ry={eye.ry} fill={eye.pupil.fill} />
            {eye.pupil.inner && <ellipse cx={eye.cx} cy={eye.cy} rx={eye.pupil.inner.rx} ry={eye.pupil.inner.ry} fill={eye.pupil.inner.fill} />}
            {eye.pupil.hi > 0 && <circle cx={eye.cx - eye.rx * 0.32} cy={eye.cy - eye.ry * 0.42} r={eye.pupil.hi} fill="#fff" />}
            {eye.glint && <ellipse cx={eye.glint.cx} cy={eye.glint.cy} rx={eye.glint.rx} ry={eye.glint.ry} fill={eye.glint.fill} />}
          </g>
        </g>)}
        <g className="stock-lids">
          {rig.eyes.boxes.map((eye, i) => <ellipse key={i} className="stock-lid" cx={eye.cx} cy={eye.cy} rx={eye.lid.rx} ry={eye.lid.ry} fill={eye.lid.fill} />)}
        </g>
        <path className="rig-happy-eyes" d={rig.eyes.arcs.d} stroke={rig.eyes.arcs.stroke} strokeWidth={rig.eyes.arcs.width} strokeLinecap="round" />
      </svg>}
    </span>
  </span>;
}
