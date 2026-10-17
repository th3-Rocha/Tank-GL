# PresetGrid

Os 36 presets da Tank-G em grade 9 × 4 (bancos 1–9, letras A–D), igual à numeração do pedal (`1A` … `9D`).

- Props: `presets` (`{ id, name, color, modified, ir }`), `value`/`onSelect` ou `defaultValue`, `activeId` (o que está tocando no pedal), `query` (apaga as que não batem), `onActivate` (duplo clique / Enter).
- Setas navegam pela grade. O preset ativo no pedal tem ▶ verde, o modificado tem ponto amarelo.
