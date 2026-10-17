# Tank-G no Linux: mapeamento e plano do app

Oct 17, 2026 · @Murilo Rocha

## O que o software oficial faz

O editor oficial (M-EFCS, só Windows) cobre IR, modelos de amp, presets e uma
cadeia fixa de seis blocos. O app novo precisa fazer tudo isso.

**IR (cabinet)**
- Abrir arquivo de IR (.wav)
- Escolher o slot de destino (8 slots, ver `PROTOCOLO.md` §2.1)
- Gravar a IR no slot do pedal
- IR Lv: nível de saída da IR
- Restaurar AMPs e IRs de fábrica

**AM2Data (modelos de amp)**
- Abrir arquivo .am2data, que são modelos de amp externos (ex.: BOSS_Crunch_IR2)
- Selecionar e gravar num slot de amp (9 slots)
- Ou seja: dá pra trocar amps, não só IR

**Global**
- Bypass
- Loopback (retorno do áudio do PC pela interface USB)
- Restaurar presets de fábrica

**Presets** (36, no formato 1A…12C)
- Selecionar preset
- Cor do LED do preset
- Salvar em outro slot (Save preset to)
- Trocar dois presets de lugar (Swap)
- Importar/exportar o preset atual
- Importar/exportar todos os presets

**Cadeia de efeitos**

| Bloco | Parâmetros |
|---|---|
| IR CAB | slot da IR |
| Noise Gate | nível |
| Reverb | on/off, Mix, Decay |
| Delay | on/off, Time, Mix |
| Mod | on/off, Speed, FX (tipo) |
| Amp | on/off, Volume, Bass, Middle, Treble, Gain, Amp Type |

## Dá pra instalar mais IR?

Dá pra substituir, não pra adicionar. O pedal tem **9 slots de amp e 8 de IR**
(confirmado pelos bancos de fábrica dentro do M-EFCS), e o software só
sobrescreve um slot por vez.

O app novo contorna isso tratando os slots como cache de uma biblioteca
ilimitada no PC:
- Você mantém centenas de IRs e amps no computador.
- Cada preset aponta para a IR pelo arquivo (hash), não pelo número do slot.
- Ao carregar um preset, o app confere se a IR já está no pedal. Se não
  estiver, grava no slot usado há mais tempo e ajusta o preset.

Gravar IR leva tempo e desgasta a flash. Por isso entra um modo "show", que
fixa no pedal as IRs do setlist e não troca nada ao vivo.

## O que o app novo entra a mais

**Biblioteca de IR e amps**
- Arrastar uma pasta inteira: o app indexa todos os .wav e .am2data
- Busca fuzzy, tags (falante, mic, tamanho de caixa) e favoritos
- Conversão automática para o formato do pedal (provável: 2048 amostras int24 mono a 44,1 kHz; a confirmar na captura)
- Pré-escuta antes de gravar: a IR é aplicada num DI de guitarra no próprio PC, sem gastar flash
- Detecção de duplicadas por hash

**Presets densos**
- Visão em tabela: 36 presets × todos os parâmetros, ordenável, com edição de várias linhas de uma vez
- Paleta de comandos (Ctrl+K): busca única em presets, IRs, amps e parâmetros
- Biblioteca ilimitada de presets no PC; o pedal recebe 36 por vez
- Setlists: lista ordenada de presets sincronizada para os slots, com IRs e amps
- Diff lado a lado entre dois presets
- Undo/redo e histórico de versões por preset
- Backup completo do pedal com um clique (presets, IRs e amps)

**Integração com o Linux**
- CLI (`tankg preset 5`, `tankg ir load ~/irs/v30.wav`) para binds do Hyprland
- Monitor MIDI para debug e mapeamento

## Capturas necessárias

Ver o checklist detalhado e o método (Wine + usbmon, sem VM) em
[`CAPTURAS.md`](CAPTURAS.md).
