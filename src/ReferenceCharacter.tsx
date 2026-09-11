import { useId } from "react";
import type { CharacterId } from "./characters";

// Totoro keeps a hand-drawn SVG rig: Ghibli publishes scene stills only, and
// every still overlaps the character with props or other cast members, so no
// clean full-body cutout exists. Geometry follows the official stills indexed
// in docs/2026-09-10-character-references.md; every other cartoon character
// renders from official-art cutout rigs in StockCharacter.
export default function ReferenceCharacter({ character }: { character: CharacterId }) {
  const prefix = useId().replace(/:/g, "");
  const paint = (name: string) => `url(#${prefix}-${name})`;
  return <svg className="reference-character" data-rig={character} viewBox="0 0 200 220" aria-hidden="true" fill="none" strokeLinecap="round" strokeLinejoin="round">
    <defs>
      <linearGradient id={`${prefix}-grey`} x2="1" y2=".5"><stop stopColor="#90908f"/><stop offset="1" stopColor="#6f737b"/></linearGradient>
    </defs>
    {character === "totoro" && <g stroke="#4f5058" strokeWidth="1.8">
      <g className="rig-ear rig-ear-left"><path d="M65 51 63 22 72 3 77 24 77 48Z" fill={paint("grey")}/></g>
      <g className="rig-ear rig-ear-right"><path d="M124 47 128 20 140 3 139 29 137 52Z" fill={paint("grey")}/></g>
      <path d="M62 47Q78 38 98 41 124 37 143 54L145 60 151 61 152 68Q171 98 177 144 185 195 149 206L61 206Q24 192 26 159 27 108 49 68L49 61 55 61 56 52Z" fill={paint("grey")}/>
      <path d="M57 108Q99 84 141 113 166 146 148 200 126 211 71 203 38 177 48 139Z" fill="#e9e0c7"/>
      <g className="rig-foot rig-foot-left"><path d="m53 198-6 9 24 2 6-8" fill="#73747a"/></g><g className="rig-foot rig-foot-right"><path d="m132 201 9 8 21-3-10-9" fill="#73747a"/></g>
      <g stroke="#969086" strokeWidth="4"><path d="m65 123 7-9 8 7m16-1 7-9 9 9m14 7 7-8 7 10m-77 15 7-10 8 8m15 7 7-11 8 8m15 7 7-11 8 12"/></g>
      <g className="rig-arm rig-arm-left"><path d="M45 99Q27 118 27 155 28 177 39 180 53 163 57 122" fill="#7b7b80"/><path d="m30 177 1 12 4-9m3-2 3 12 1-14"/></g>
      <g className="rig-arm rig-arm-right"><path d="M149 100Q170 112 174 156 177 178 164 182 151 169 144 122" fill="#74767e"/><path d="m165 180 4 13 2-14m-10 0 2 13 2-12"/></g>
      <g className="rig-face"><g className="rig-eye"><ellipse cx="75" cy="68" rx="9" ry="10" fill="#f5f0df"/><g className="rig-pupil"><ellipse cx="76" cy="68" rx="2.7" ry="4.5" fill="#24292d" stroke="none"/></g></g><g className="rig-eye"><ellipse cx="123" cy="69" rx="9" ry="10" fill="#f5f0df"/><g className="rig-pupil"><ellipse cx="122" cy="69" rx="2.7" ry="4.5" fill="#24292d" stroke="none"/></g></g>
        <path className="rig-happy-eyes" d="m68 69q7-8 13 0m35 1q7-8 13 0" strokeWidth="3"/>
        <path d="M87 73Q99 65 111 75 98 80 87 73Z" fill="#333c43"/>
        <path d="m59 76-31-13m29 20-36-3m37 10-36 9m119-20 31-13m-29 19 37-4m-35 10 35 7" stroke="#34363d"/>
        <path d="M61 90Q98 99 141 91 128 109 98 110 72 108 61 90Z" fill="#f9f3df"/><path d="m74 94 2 9m15-7v12m15-12-1 12m15-13-2 10m12-12-1 6" strokeWidth="1.2"/>
      </g>
    </g>}
  </svg>;
}
