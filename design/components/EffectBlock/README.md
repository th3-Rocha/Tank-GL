# EffectBlock

Bloco de um efeito com seus knobs: barra âmbar em cima, título em caixa alta e toggle de liga/desliga.

- Props: `title`, `subtitle`, `enabled`/`onToggle` ou `defaultEnabled` (sem nenhum deles, o bloco não tem toggle), `children` (os `Knob`).
- Desligado: título ganha "· OFF", a barra fica cinza e os knobs perdem o brilho, mas continuam editáveis.
