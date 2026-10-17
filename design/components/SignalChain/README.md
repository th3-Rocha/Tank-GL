# SignalChain

A cadeia de sinal desenhada como hardware: jack IN → GATE → amp (cabeçote com painel, knobs e lâmpada) → pedais MOD, DELAY, REVERB (com LED, knobs e footswitch) → caixa 2×12 com os cones da IR → jack OUT, ligados por cabos com o sinal correndo.

- Props: `nodes` (`{ id, kind, label, on, sub, led, amount }`, com `kind` `gate` · `amp` · `stomp` · `cab`), `selected`/`onSelect` ou `defaultSelected`, `onToggle(id, on)`, `signal` (falso = pedal desconectado, cabos apagados).
- Clicar num módulo o seleciona (contorno âmbar com brilho); clicar no footswitch liga ou desliga sem mudar a seleção.
- Setas esquerda/direita andam pela cadeia; Enter seleciona.
- Mostre os knobs do módulo selecionado logo abaixo (veja `PresetEditor`).
