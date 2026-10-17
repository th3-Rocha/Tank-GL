# Button

O bloco cinza `#353535` com texto branco em negrito do editor oficial. Variantes:

- `secondary` (padrão): a maioria das ações ("Open IR File", "Swap", "Importar pasta").
- `primary` (âmbar, brilha no hover): no máximo um por tela, para a ação que grava no pedal.
- `ghost`: ações secundárias em barras e linhas (Reverter, pré-escutar).
- `danger`: restaurar de fábrica e apagar. Peça confirmação antes.
- `color`: só o botão "Color" do LED do preset (`chartreuse`).

Props: `variant`, `size` (`sm` · `md` · `lg`), `icon`, `iconRight`, `kbd` (atalho mostrado no botão), `block`, `disabled` e os atributos de `<button>`. Botão só com ícone exige `aria-label`. Rótulo com verbo no infinitivo: "Gravar no pedal".
