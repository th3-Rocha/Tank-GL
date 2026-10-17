# CommandPalette

A busca global do app (Ctrl K): presets, IRs, amps, parâmetros, setlists e ações num lugar só, com busca aproximada ("v30 57", "bkp", "2d").

- Props: `items` (`{ id, kind, title, subtitle, meta, icon, color, keywords }`), `onSelect(item)`, `onClose`, `open`, `inline` (sem véu, para embutir), `defaultQuery`, `placeholder`, `limitPerGroup`, `maxHeight`.
- `kind` agrupa: `action`, `preset`, `ir`, `amp`, `param`, `setlist`. Presets com `color` mostram o LED.
- Coloque o id do preset (`2D`) e o nome do arquivo em `keywords` para eles serem achados também.
- Setas navegam, Enter escolhe, Esc limpa e depois fecha.
