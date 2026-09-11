import { createRoot } from "react-dom/client";
import PetFeatures from "./PetFeatures";
import { characters } from "./characters";
import "./App.css";
import "./characters.css";
import "./cartoonCharacters.css";

const previewIds = ["shinchan", "spongebob", "cinnamoroll", "kuromi", "pooh", "pikachu", "doraemon", "kirby", "hello_kitty"];
document.body.style.overflow = "auto";
document.documentElement.style.overflow = "auto";
const states: { key: string; label: string; props: Record<string, string> }[] = [
  { key: "idle", label: "常态", props: {} },
  { key: "wave", label: "挥手", props: { "data-reaction": "wave" } },
  { key: "pet", label: "摸摸", props: { "data-reaction": "pet" } },
  { key: "lift", label: "拖起", props: { "data-lifted": "true" } },
];
createRoot(document.getElementById("root")!).render(
  <div style={{ display: "flex", flexWrap: "wrap", gap: "28px 22px", padding: "30px", background: "#efe9dd" }}>
    {characters.filter(item => previewIds.includes(item.id)).map(item => (
      <figure key={item.id} style={{ margin: 0, textAlign: "center", fontFamily: "sans-serif" }}>
        <div style={{ display: "flex", gap: 14 }}>
          {states.map(state => (
            <div key={state.key} style={{ textAlign: "center" }}>
              <div style={{ position: "relative", width: 130, height: 140, background: "#fffdf8", borderRadius: 10, boxShadow: "0 3px 10px #00000014", zoom: 0.72 }}>
                <div className="tomato" data-character={item.id} {...state.props}><PetFeatures character={item.id} /></div>
              </div>
              <small style={{ color: "#8a8070" }}>{state.label}</small>
            </div>
          ))}
        </div>
        <figcaption style={{ color: "#5a5245", marginTop: 8 }}>{item.name}</figcaption>
      </figure>
    ))}
  </div>
);
