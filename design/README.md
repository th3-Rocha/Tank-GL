# Tank Studio design

Tank Studio is an unofficial Linux editor for the M-VAVE Tank-G pedal. This
folder holds the original design tokens and the icon set.

> The React components that lived here belonged to the old Tauri + React
> front end and were removed with it (still in git history). The egui app
> (`app/`) defines its own palette in `app/src/theme.rs`: dark teal surfaces
> and the cyan of the Tank-G's body as the accent. `tokens.json` here is the
> earlier palette, modelled on the official editor (purple gradient, graphite
> cards), kept as reference.

## Files

| File | Contents |
|---|---|
| `tokens.json` | Source of truth for the tokens: colors, type, spacing, radii, sizes, shadows, each with a usage note |
| `tokens.css` | The same tokens as CSS variables, generated from `tokens.json` |
| `assets/Icons/` | 36 stroke icons (see its README) |

## Principles

- **Looks like the Tank-G.** UPPERCASE labels as printed on the pedal, the same parameter names (GAIN, TREBLE, IR CAB), and each block in the pedal's own color.
- **The knob is the main gesture.** Every continuous parameter is a knob: drag, scroll or arrow keys, Shift for fine steps, double-click to go back to the saved value.
- **Honest hardware.** The app shows what is on the pedal, and what changed but was not saved ("Modified · not saved").
- **Dense, but readable in the dark.** White or muted text on dark surfaces, never dark gray on gray.

## Voice and text

- Interface in English. Parameter names stay as on the pedal, in uppercase: `GAIN`, `TREBLE`, `MOD FX`, `NOISE GATE`.
- Buttons say the action and its object: "Save to 5C", "Play on the pedal", "Back up presets". No bare "OK" or "Send".
- Numbers carry a unit when there is one: "46 ms", "slot 3", "8 KB".
- Status is always a word, never color alone: "Connected", "Modified", "Saved".
- No emoji, no exclamation marks. Empty states say the next step: "Connect the pedal to see the preset."

## States

- **Keyboard focus:** a visible outline on every control.
- **Selected:** soft accent background plus an accent border.
- **Off / bypass:** the block's title reads "· OFF", its knobs lose their glow and the whole pedal is dimmed.
- **Modified:** a warning tag "Modified · not saved".
- **Disabled:** faint text, no glow.
