# Icon

Desenha um dos 36 ícones de traço do sistema pelo nome. Passe `name` (veja a lista em `Tank.ICON_NAMES`) e, opcionalmente, `size` (padrão 20) e `title`.

- O ícone herda `color` do pai (`currentColor`); pinte pelo contêiner.
- Sem `title` ele é decorativo (`aria-hidden`). Botão só com ícone precisa de `aria-label` no botão.
- Use 16px em linhas densas e botões `sm`, 18–20px no resto.
