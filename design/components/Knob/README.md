# Knob

O controle principal do app: anel de LEDs, arco âmbar com brilho, capa metálica recartilhada e ponteiro. Arraste para cima ou para a direita, role a roda, ou use as setas; `Shift` faz ajuste fino, duplo clique volta ao `resetValue` (ou `defaultValue`), `Delete` também.

- Props: `label` (em caixa alta, como no pedal), `value`/`onChange` ou `defaultValue`, `min`, `max`, `step`, `size` (`sm` · `md` · `lg`), `unit`, `format(v)`, `valueLabels` (nomes para valores discretos, ex. tipos de amp), `ticks`, `resetValue`, `disabled`.
- Com `valueLabels` ou `ticks`, o anel mostra um LED por posição e o atual fica branco.
- Ao arrastar, um balão com o valor aparece acima do knob.
- Dentro de um `EffectBlock` em bypass, o arco e o ponteiro perdem o brilho e ficam `ink-muted`.
- Use `lg` só para os dois ou três knobs principais de um bloco.
