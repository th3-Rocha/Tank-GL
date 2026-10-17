Tank Studio é o editor não oficial da pedaleira M-VAVE Tank-G para Linux. A interface imita o editor oficial: fundo roxo-escuro, cartões grafite, botões cinza `#353535` com texto branco em negrito, e o âmbar `#FFAA00` dos knobs como a única cor de destaque. Em cima disso entram profundidade, brilho e uma cadeia de sinal ilustrada, porque o app é usado no palco e precisa ser lido de longe.

## Princípios

- **Parece a Tank-G.** Quem já usou o editor oficial reconhece tudo de cara: as mesmas cores, os rótulos em CAIXA ALTA como na serigrafia do pedal, os mesmos nomes de parâmetro (GAIN, RVB DECAY, IR CAB).
- **O knob é o gesto principal.** Todo parâmetro contínuo é um `Knob`. Fader só para níveis em dB (IR LV, pré-escuta).
- **O som é uma cadeia que dá para ver.** O editor mostra o caminho do sinal como hardware: GATE → AMP → MOD → DELAY → REVERB → CAB/IR, ligados por cabos. Clicar num módulo abre os knobs dele.
- **Tudo é pesquisável.** Presets, IRs, amps, parâmetros e ações passam pela mesma busca (`CommandPalette`, Ctrl K).
- **Denso, mas legível no escuro.** Linhas de 32px, texto branco ou `ink-muted`, nunca cinza-escuro em cima de cinza.
- **Hardware honesto.** O app mostra o que está no pedal e o que está só no PC (`slot` vs "só no PC"), e o que mudou e não foi gravado (`Modificado`).

## Voz e texto

- Interface em português do Brasil. Nomes de parâmetro ficam como no pedal, em inglês e caixa alta: `GAIN`, `TREBLE`, `MOD FX`, `NOISE GATE`.
- Botões com verbo no infinitivo e o objeto: "Gravar no pedal", "Importar pasta", "Restaurar de fábrica". Nada de "OK" ou "Enviar" solto.
- Números sempre com unidade quando há uma: "−6 dB", "48k · 1024", "slot 3".
- Status sempre com palavra, nunca só cor: "Conectado", "Modificado", "Salvo no pedal".
- Sem emoji, sem exclamação. Mensagens de vazio dizem o próximo passo: "Biblioteca vazia. Arraste uma pasta de .wav para cá."

## Cor

Um único tema, **Tank-G**, copiado do editor oficial.

- **Fundo da janela:** degradê de `ground-top` (`#210031`) para `ground-bottom` (`#0f0016`). Texto nunca fica direto no degradê; vai num cartão.
- **Cartões e painéis:** `surface-card` para os cartões grandes (IR, AM2Data, cabeçalho do editor), `surface-raised` para blocos de efeito, `surface` (`#151515`) para poços: listas, cadeia de sinal, busca, monitor MIDI.
- **Controles:** `control` (`#353535`) é o fundo de botão e a trilha vazia dos knobs, como no oficial. Hover em `control-hover`.
- **Âmbar:** `amber` é a cor do app. Use em arco de knob, toggle ligado, seleção, nome de arquivo escolhido, botão primary. `amber-soft` é o fundo de linha ou célula selecionada. `amber-glow` só vira brilho (`drop-shadow`, `box-shadow`), nunca preenchimento.
- **Texto:** `ink` (branco) sobre qualquer superfície; `ink-muted` para rótulos de knob e metadados; `ink-faint` só para desabilitado.
- **Toggle:** ligado = `amber` com polegar `toggle-thumb`; desligado = `toggle-off` (`#c8c8c8`). Igual ao oficial.
- **Botão de cor:** `chartreuse` (`#c1c100`) aparece só no botão "Color" do LED do preset. O oficial põe texto branco nele (1.9:1, ilegível); aqui o texto é `on-chartreuse`.
- **Status:** `ok` (ciano), `warn` (amarelo), `danger` (coral), cada um com o fundo `*-soft`. Sucesso e erro diferem em matiz e em luminosidade, e sempre vêm com palavra ou ícone.
- **Hardware:** `display-bg`, `display-on` e `display-off` no display de 7 segmentos. `led-*` são as oito cores de LED de preset e dos pedais da cadeia; aparecem só como luz.
- **Ilustrações:** `tolex`, `grille`, `metal` e `cable` desenham amp, caixa, ferragens e cabos da `SignalChain`.

## Tipografia

- **Barlow Condensed** (`condensed`) para tudo que imita a serigrafia: nome do preset (`display`), títulos de bloco (`label-lg`), rótulos de knob e de coluna (`label`, sempre em caixa alta).
- **Barlow** (`sans`) para a interface: `button` em negrito (700) como os botões oficiais, `body` para linhas e textos, `title` em painéis.
- **JetBrains Mono** (`mono`) para números e bytes: `value` sob os knobs, `readout` no balão de arraste, `code` em nomes de arquivo e mensagens MIDI. Use `tabular-nums` para os valores não pularem.

## Espaço, cantos e tamanho

- Grade de 4px: `space-2` dentro de controles, `space-4` de padding de painel, `space-6` entre painéis.
- Cantos quase retos nos controles, como no oficial: `radius-xs` (3px) em botões, tags e selects; `radius-md` (10px) em painéis; `radius-lg` só na paleta de comandos.
- Botão padrão `ctrl-md` (32px). Linhas densas `row-dense` (32px). Knobs em três tamanhos: `knob-sm` 60, `knob-md` 88, `knob-lg` 116. Os dois principais do AMP (GAIN, VOLUME) usam `lg`.

## Profundidade e brilho

- Painéis levam `shadow-panel`. A paleta leva `shadow-pop` mais um halo âmbar.
- O brilho é o que diz "isto está ativo". Arco de knob, ponteiro, LED de pedal, cabo com sinal, toggle ligado e nó selecionado brilham em âmbar (`amber-glow`, `shadow-glow`). Coisas desligadas não brilham: o knob de um bloco em bypass passa para `ink-muted` sem glow, o LED do pedal apaga.
- O display de LED tem scanlines, reflexo de vidro e segmentos com brilho (`shadow-led`).
- Movimento: o sinal corre pelos cabos, os cones da caixa vibram de leve quando há som, o status "Conectando…" pulsa. Tudo para com `prefers-reduced-motion`.

## Estados

- **Foco de teclado:** 2px sólido `focus` (branco) com 2px de afastamento, em todo controle. Na cadeia de sinal, o contorno do módulo vira branco.
- **Selecionado:** fundo `amber-soft` e borda ou barra `amber`.
- **Bypass:** bloco com título "· OFF", barra superior cinza, knobs sem brilho; na cadeia o pedal fica a 50% e o LED apaga.
- **Modificado:** tag `warn` "Modificado" e ponto amarelo na célula do preset; "Gravar no pedal" só habilita aí.
- **Desabilitado:** texto `ink-faint`, sem brilho, cursor `not-allowed`.

## Iconografia

Conjunto próprio de 36 ícones de traço (24px, traço 1.75, pontas redondas), no grupo **Icons**. Nos componentes, use `Tank.Icon` com o nome; os SVGs do grupo são para outros usos e têm tinta fixa `#8f8b83`. Sem emoji. Não há logotipo: M-VAVE e Tank-G são marcas do fabricante, então o nome do app é escrito em `display`.

## Cadeia de sinal

A ordem desenhada é GATE → AMP → MOD → DELAY → REVERB → CAB/IR. É a ordem pedida para o app; a ordem real dentro da Tank-G ainda precisa ser confirmada na captura do protocolo. Os limites dos parâmetros (0–100, AMP TYPE 1–9, IR CAB 1–8) e os nomes dos tipos de amp também são provisórios até a captura.

## Usando no app

- `tokens.json` é a fonte de verdade. No Tauri ou numa página web, carregue `tokens.css` e `components/bundle.css` e use as variáveis (`var(--amber)`).
- No Rust com Slint, gere um `global Theme` a partir de `tokens.json` (uma propriedade `brush` por cor, `length` por espaço, raio e tamanho) em vez de copiar valores à mão.
- Os componentes React estão em `components/bundle.js` como `window.Tank` e esperam React 18 na página. Cada componente tem um guia em `components/<Nome>/README.md` e um preview vivo.
