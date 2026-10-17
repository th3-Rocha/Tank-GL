# SearchField

Campo de busca embutido em listas e grades (biblioteca, presets).

- Props: `value`/`onChange` ou `defaultValue`, `placeholder`, `width`, `size` (`md` · `lg`), `shortcut` (texto do atalho; `null` esconde), `hotkey` (liga Ctrl K para focar), `autoFocus`.
- `Esc` limpa; o X aparece quando há texto. Ao focar, a borda vira âmbar com brilho.
- Para busca global (presets + IRs + ações), use `CommandPalette`.
