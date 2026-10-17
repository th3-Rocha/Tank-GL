# LibraryTable

Lista densa de IRs (ou amps) no PC, como a lista preta do editor oficial, com nome em âmbar quando selecionado.

- Props: `items` (`{ id, name, file, tags, format, slot, favorite }`), `query`, `tags` (filtro; todas precisam bater), `selected`/`onSelectionChange`, `onPreview`, `onSend`, `onToggleFavorite`, `playingId`, `maxHeight`, `noun`, `footerActions`.
- Clique seleciona; Ctrl/Cmd soma; Shift seleciona intervalo; setas andam; Espaço marca; Enter pré-escuta.
- Coluna SLOT diz se a IR está no pedal (número âmbar) ou "só no PC".
- Combine com `SearchField` e tags filtráveis acima, e a ação de gravar no `footerActions`.
