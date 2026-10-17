# LedDisplay

Display de 7 segmentos com moldura, scanlines, reflexo de vidro e brilho. Mostra o preset atual (`2D`), testes (`88`), sem pedal (`--`) ou palavras curtas (`SAVE`).

- Props: `value` (texto), `digits` (preenche à esquerda), `height` (padrão 40), `color` (troca `display-on`, ex. `var(--led-green)` para "gravado"), `caption`, `label` (texto para leitores de tela).
- Letras sem forma em 7 segmentos (K, M, W, X…) saem em branco: escolha outra palavra.
