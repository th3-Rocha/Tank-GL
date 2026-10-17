# SegmentedControl

Escolha exclusiva entre 2–5 opções sempre visíveis: transporte (USB / Bluetooth), tela (Editor, Biblioteca, Presets, Setlists).

- Passe `options` (`{ value, label, icon }`), `value`/`onChange` ou `defaultValue`, e `label` para leitores de tela.
- O item ativo fica em `control` com uma linha âmbar embaixo e o ícone brilhando.
- Setas do teclado trocam a opção. Mais de 5 opções: use `Select`.
