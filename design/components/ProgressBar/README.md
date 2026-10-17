# ProgressBar

Progresso de operações com o pedal: gravar IR, ler presets, backup.

- Props: `label`, `value` (0–1; omita para indeterminado), `detail` (bytes, arquivo, contagem em mono), `tone` (`amber` · `ok` · `danger`).
- A barra âmbar tem listras e brilho enquanto grava; troque para `ok` ao terminar.
- Gravação na flash não pode ser cancelada no meio: diga isso no `detail` se for o caso.
