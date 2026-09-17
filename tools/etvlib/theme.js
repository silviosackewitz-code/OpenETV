/* The look the user chose, in an open window: light or dark, and the accent.
 *
 * The server writes both into every page it serves (`racelib/theme.py`), so a
 * window starts right. This module keeps it right while the window is open:
 * `followLook()` applies the preferences whenever they are read or saved
 * (`prefs.js`). Deriving the accent's roles stays in the backend — it sends
 * the finished rules (`accent_css`), this module only puts them in place.
 *
 * A canvas does not repaint by itself when colours change: `onLookChange(fn)`
 * calls *fn* after the theme or the accent changed, by the user's choice or
 * because the system switched between light and dark.
 *
 * `mountAppearance(el)` builds the controls to choose both.
 *
 * Nothing runs on import.
 */

import { esc } from "./common.js";
import { onPrefs, savePrefs, watchPrefs } from "./prefs.js";

//: The element the server put the accent's rules in (`theme.ACCENT_STYLE_ID`).
const ACCENT_STYLE = "accentRoles";
const THEMES = [["auto", "Auto", "Follow the system"], ["light", "Light", ""], ["dark", "Dark", ""]];
const redraws = [];
let following = false;

//: Call *fn* after the look changed.
export function onLookChange(fn){ redraws.push(fn); }

//: Put *prefs* (`theme`, `accent_css`) in place. Returns whether anything changed.
export function applyLook(prefs){
  const root = document.documentElement;
  const theme = prefs.theme === "auto" ? undefined : prefs.theme;
  let style = document.getElementById(ACCENT_STYLE);
  if (!style){
    // After everything else in <head>, as the server places it after theme.css.
    style = Object.assign(document.createElement("style"), { id: ACCENT_STYLE });
    document.head.append(style);
  }
  const changed = root.dataset.theme !== theme || style.textContent !== prefs.accent_css;
  if (theme) root.dataset.theme = theme; else delete root.dataset.theme;
  if (style.textContent !== prefs.accent_css) style.textContent = prefs.accent_css;
  if (changed) for (const fn of redraws) fn();
  return changed;
}

//: Keep this window in the chosen look. Once per page; returns the first read.
export function followLook(){
  if (following) return Promise.resolve(null);
  following = true;
  onPrefs(applyLook);
  matchMedia("(prefers-color-scheme:dark)").addEventListener("change",
    () => { for (const fn of redraws) fn(); });
  return watchPrefs();
}

/* The controls: Auto / Light / Dark, the offered accents and a free colour.
 * Built into *el*; they show what the preferences say and save every change
 * at once — there is nothing to confirm, the window is the preview. */
export function mountAppearance(el){
  el.innerHTML = `
    <div class="field"><span class="field-name" id="lookThemeName">Theme</span>
      <div class="segmented" role="radiogroup" aria-labelledby="lookThemeName">${
        THEMES.map(([value, name, title]) =>
          `<label title="${esc(title)}"><input type="radio" name="lookTheme" value="${value}"><span>${name}</span></label>`
        ).join("")}</div></div>
    <div class="field"><span class="field-name" id="lookAccentName">Accent</span>
      <div class="swatches" role="group" aria-labelledby="lookAccentName"></div></div>`;
  const swatches = el.querySelector(".swatches");
  const save = async values => {
    const reply = await savePrefs(values);
    if (!reply.ok) console.warn("could not keep the look:", reply.error);
  };
  el.addEventListener("change", e => {
    if (e.target.name === "lookTheme") save({ theme: e.target.value });
    if (e.target.type === "color") save({ accent: e.target.value });
  });
  swatches.addEventListener("click", e => {
    const swatch = e.target.closest("button[data-colour]");
    if (swatch) save({ accent: swatch.dataset.colour });
  });
  onPrefs(prefs => {
    for (const radio of el.querySelectorAll('input[name="lookTheme"]'))
      radio.checked = radio.value === prefs.theme;
    const own = !prefs.accent_presets.some(p => p.colour === prefs.accent);
    swatches.innerHTML = prefs.accent_presets.map(p =>
      `<button type="button" class="swatch" data-colour="${esc(p.colour)}" style="--swatch:${esc(p.colour)}"`
      + ` aria-pressed="${p.colour === prefs.accent}" title="${esc(p.name)}" aria-label="${esc(p.name)}"></button>`
    ).join("")
      + `<label class="swatch own" style="--swatch:${esc(prefs.accent)}" data-pressed="${own}" title="Another colour">`
      + `<input type="color" value="${esc(prefs.accent)}" aria-label="Another colour"></label>`;
  });
}
