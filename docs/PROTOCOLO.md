# Protocolo do Tank-G: o que sabemos e o que falta confirmar

> Estado em 2026-10-17. O Tank-G ainda **não foi plugado nem capturado**. Tudo
> aqui vem de três fontes: (1) o binário do editor oficial `M-EFCS.zip`,
> (2) a engenharia reversa publicada de outros pedais M-VAVE que usam o mesmo
> editor e (3) inferência a partir dessas duas.
>
> Marcadores usados no texto:
> - ✅ **verificado**: está no binário do M-EFCS ou foi medido em outro pedal da família
> - 🟡 **hipótese forte**: o padrão é o mesmo em 3 ou mais pedais M-VAVE
> - ❓ **a capturar**: só a captura no Tank-G resolve

---

## 1. Resumo

1. O Tank-G conversa por **USB-MIDI class-compliant com SysEx**. No Linux ele
   deve aparecer como porta ALSA sem driver nenhum (`snd-usb-audio`). Ele
   também é uma interface de áudio USB, e é daí que vem o recurso Loopback.
2. Todos os pedais M-VAVE recentes usam o mesmo frame bruto
   **`00 59 | cmd | len u24 | corpo | checksum`**. No USB esse frame vai
   empacotado em 7 bits dentro de `F0 … F7`. Como `00 59` empacotado vira
   `00 32`, toda mensagem começa com `F0 00 32`.
3. O editor oficial é um app **Flutter** (`m_efcs`) e tem código específico do
   Tank-G (`data/products/tank_g/structs.dart`, `effect_params.dart`,
   `transport/midi/sysex_codec.dart`). Ele traz embutidos os **bancos de
   fábrica do Tank-G**: 9 slots de amp e 8 de CAB.
4. O **Tank Mini** (irmão menor) tem o protocolo inteiro documentado: preset,
   knobs, toggles, dump de 92 bytes e save na flash. Ele usa exatamente o
   mesmo endereçamento de memória visto no BlackBox. A hipótese de trabalho é
   que o Tank-G segue esse mesmo modelo.
5. O **upload de IR** (registro `CAB\0`, 2048 amostras int24 a 44,1 kHz) já foi
   reproduzido **byte a byte** no MK-300 e no BlackBox. O Tank-G tem slots de
   CAB do mesmo tipo.
6. O **M-EFCS roda no Linux pelo Wine** com um patch de 1 byte. Com isso dá pra
   capturar direto no Linux com `usbmon`, sem VM (ver `CAPTURAS.md`).

---

## 2. Evidências do binário oficial (`M-EFCS.zip`, build de 2026-10-10)

| Item | Achado | |
|---|---|---|
| Framework | Flutter/Dart AOT (`data/app.so`), plugins `flutter_midi_command_windows`, `universal_ble` | ✅ |
| Transporte | `transport/midi/sysex_codec.dart`, classes `SysExCodec` e `SysExStreamDecoder`; também BLE-MIDI (`BLEMidiDevice`) | ✅ |
| Código do Tank-G | `products/tank_g/{config,data,effect_params,structs}.dart`, `ui/products/tank_g_view.dart`, `buildTankGEffectParams` | ✅ |
| Strings de função | `setLoopbackEnabled`, `swapPreset`, `SwapPresetToDialog`, `presetSaved`, `am2data`, `ImportCabWidget`, `utils/wave_utils.dart` | ✅ |
| Bancos de fábrica | `flutter_assets/lib/resources/bin/tankG/` (detalhes abaixo) | ✅ |

### 2.1 Bancos de fábrica embutidos

`tankG_am3_*` e `tankG_am4_*` são duas gerações de firmware. A `am3` usa amps
no formato `AM2`, a `am4` usa `AM4`. Também existe `stationG_*`, uma variante
do mesmo produto.

**`*_ampCab.bin`** tem 212 992 bytes (`0x34000`), ou seja, 17 registros:

| Offset | Tamanho do slot | Conteúdo (am4) |
|---|---|---|
| `0x00000` + n·`0x4000` | 16 KiB | **9 amps**: `AM4\0` + nome (14 B) + floats (rede neural). Fábrica: `1J120_CL`, `2J900_CL`, `3VAVE_TS`, `4RAT_OD`, `5VAVE_HTS`, `6VAVE_DS`, `7MS_GV2`, `8J900_HV`, `9MS_HIGAIN` |
| `0x24000` + n·`0x2000` | 8 KiB | **8 CABs**: `CAB\0` + nome (14 B) + `02` + nível + amostras. Fábrica: `1AC-SeVin`, `2JVM_1960_57`, `3MESA_412`, `4WANGS112`, `5VOX_AC30`, `6HIFI_YAMAHA`, `7EGNL_412`, `8V30_MD421` |

O cabeçalho do AMP é igual ao do MK-300 (`64 c8 3c 1e …` seguido de 9 floats
de constantes). Um `.am2data` é então **um registro de amp `AM2`** e cabe num
slot de 16 KiB. No `am3`, o CAB ocupa cerca de 6 204 bytes, perto dos 6 164
bytes do BlackBox e do MK-300 (`CAB\0` + nome + 2 bytes + 2048 × int24).

Fica respondida a pergunta "dá pra instalar mais IR?": são **9 slots de amp e
8 de IR fixos**. Isso valida o desenho de "slots como cache" do plano.

**`*_preset.bin`** tem 758 bytes e começa com o magic `a5 5a`. Os registros
têm cerca de 21 bytes, com nomes que parecem ofuscados (`b9CTL`, `GENUL`…).
Não é o formato cru de 92 bytes do Tank Mini; deve ser comprimido ou
codificado. ❓ Decodificar pelo `structs.dart` ou cruzando com o dump.

---

## 3. Camada USB

| Item | Valor esperado | |
|---|---|---|
| Classe | USB-MIDI 1.0 + USB Audio 2in/2out 44,1 kHz/24 bit; driver `snd-usb-audio` | 🟡 (IR Box, BlackBox e MK-300 são assim) |
| VID:PID | Família SINCO `4353:xxxx` (IR Box = `4353:4B4D`) ou Jieli `4c4a:xxxx` (BlackBox = `4c4a:c755`) | ❓ `lsusb` |
| Nome da porta | `USB-Midi` (Tank Mini, IR Box) ou `USB Composite Device` (MK-300, BlackBox) | ❓ `aconnect -l` |
| Linux | O ALSA sequencer convive com o PipeWire. `midir` usa o ALSA seq | ✅ (BlackBox no Fedora/PipeWire) |

Para achar o pedal, casar com o nome da porta e, se possível, também com o
VID:PID lido do sysfs. Pedais Jieli genéricos se chamam todos "Composite".

---

## 4. Enquadramento (frame bruto, 8 bits)

```
off  0  1   2     3..5           6 .. 6+len-1   6+len
     00 59  cmd   len (u24 LE)   corpo          cs
```

- **Checksum:** `cs = (~soma(corpo)) & 0xFF`, sem os 6 bytes de cabeçalho.
  Corpo vazio dá `FF`. 🟡 (IR Box verificado; Tank Mini e MK-300 dão o mesmo
  resultado.) O pedal **descarta em silêncio** frames com checksum errado. ✅ (MK-300)
- **Comandos:**

| cmd | Nome | Uso |
|---|---|---|
| `00` | ACK/status | resposta a toda escrita. `00 59 00 01 00 00 00 FF` = ok, status `01` = erro |
| `11` | identidade | retorna nome e firmware (ex.: `TANK MINI_009`, `IR-BOX_010`) |
| `12`,`13`,`17`,`1B`,`20` | queries | várias ficam sem resposta, então usar timeout curto |
| `21` | **ERASE** | ⛔ **PROIBIDO.** O app tem esse comando; nosso código deve recusar enviar em qualquer caso |
| `22` | WRITE | `tipo(1) addr(u32 LE) len(u24 LE) dados` |
| `23` | READ | pedido: `tipo addr len`. Resposta: eco do pedido + dados |

### 4.1 Empacotamento 7-bit para o USB

O frame é lido como um fluxo de bits contínuo, LSB primeiro, e sai em
pedaços de 7 bits. Não é o esquema "1 byte de MSBs a cada 7 bytes" comum em
outros fabricantes.

```
accum = 0; nbits = 0
for b in frame:
    accum |= b << nbits; nbits += 8
    while nbits >= 7: emit(accum & 0x7F); accum >>= 7; nbits -= 7
emit(accum & 0x7F)     # byte final de flush, sempre emitido
```

`n` bytes viram `n + n//7 + 1`. Para desempacotar, use `n_raw = 7·len/8` e
corte pelo `len` do cabeçalho. **Nunca** remova zeros do final, porque o
checksum pode ser `00`.

Vetores de teste (IR Box e Cube Baby):

| Bruto | No fio |
|---|---|
| `00 59 11 00 00 00 FF` (query identidade) | `F0 00 32 45 00 00 00 40 7F 00 F7` |
| `00 59 00 01 00 00 00 FF` (ACK ok) | `F0 00 32 01 08 00 00 00 00 7F 01 F7` |
| `00 59 22 09 00 00 05 09 00 00 80 01 00 00 01 6F` | `F0 00 32 09 49 00 00 40 02 09 00 00 00 18 00 00 00 01 5E 01 F7` |

O doc do MK-300 descreve o mesmo frame "visto do lado empacotado" (campos
`CLS/CMD/FIELD/SUB`). É a mesma coisa. Vamos sempre raciocinar no frame bruto.

No **BLE** o Tank Mini manda o frame bruto, sem SysEx e sem empacotamento.
GATT `ae40`, TX `ae41`, RX `ae42`. Fica para depois.

---

## 5. Modelo de memória (hipótese de trabalho para o Tank-G)

Leituras e escritas endereçam `tipo` + `addr u32`. No tipo `04` (DEV), o
**nibble alto do endereço** escolhe a "família". O mapa abaixo junta o
BlackBox (análise do firmware + leituras), o Tank Mini (captura) e o MK-300
(captura):

| addr | Conteúdo | Evidência | Tank-G |
|---|---|---|---|
| `0x0000_0000 + slot·92` | **Banco de presets na flash.** Registros de 92 B (`0x5C`) + trailer `PATCHEND` (10 B). BlackBox: limite `0x1CCA` = 80×92+10 | BlackBox ✅, Tank Mini ✅ (save em `id·92`) | 🟡 36×92+10 = 3 322 B |
| `0x1000_0000 + off` | **Patch atual (edit buffer)**, 92 B. Escrita aqui muda o som na hora, só em RAM | BlackBox ✅, Tank Mini ✅ | 🟡 |
| `0x2000_0000` | estado de 4 B (preset ativo?) | BlackBox ✅ | ❓ |
| `0xD000_0000` | **Loopback** (1 byte on/off) | BlackBox ✅ (comparação OFF/ON/OFF) | 🟡 |
| `0xE000_0000 + id` | **Selecionar preset** (escrever `01`) | Tank Mini ✅, IR Box ✅ | 🟡 |
| `0xA…` / `0xF…` | audition/refresh e commit/save | MK-300 ✅, IR Box ✅ | ❓ |

Também pode trocar de preset com Program Change `C0 <id>`. ✅ no Tank Mini;
o fórum do Tank-G cita PC para os 36 presets e CC para os módulos.

### 5.1 Comandos do Tank Mini traduzidos para esse modelo

O "roteamento `00 00 04`" que o doc do Tank Mini descreve é, na verdade,
`len=…, tipo=04`. E `[addr] 00 00 10` é o u32 LE `0x100000xx`, ou seja,
família 1 no offset `xx`. Fica tudo coerente:

```
Selecionar preset  00 59 22 09 00 00 | 04 | id 00 00 E0 | 01 00 00 | 01 | cs
Toggle de bloco    00 59 22 09 00 00 | 04 | TT 00 00 10 | 01 00 00 | 0/1 | cs
Trocar modelo      00 59 22 09 00 00 | 04 | MM 00 00 10 | 01 00 00 | id  | cs
Knob (u16 LE)      00 59 22 0A 00 00 | 04 | KK 00 00 10 | 02 00 00 | v 00 | cs
Ler patch atual    00 59 23 08 00 00 | 04 | 00 00 00 10 | 5C 00 00 | cs
Salvar na flash    00 59 22 64 00 00 | 04 | (id·92) u32 | 5C 00 00 | 92 B | cs
```

### 5.2 Patch de 92 bytes (layout do Tank Mini, ponto de partida)

```
0..5    ordem da cadeia (ids dos blocos)
6..7    padding
8..13   on/off por bloco
14..19  modelo por bloco
20..91  knobs: 6 blocos × 12 B (6 × u16 LE, 0..100)
```

O Tank Mini tem os blocos FX, AMP, MOD, DLY, REV e CAB. O Tank-G tem IR CAB,
Noise Gate, Reverb, Delay, Mod e Amp. São os mesmos seis, com o Gate no lugar
do FX. ❓ Falta confirmar ordem, endereços, onde fica a **cor do LED**, o
**IR Lv** e se os knobs vão de 0 a 100.

---

## 6. Upload de IR e de amp

Reproduzido byte a byte no MK-300 (jpfaria/mvave) e no BlackBox
(Bulrok/irload.py), a partir do código do próprio M-EFCS
(`utils/wave_utils.dart`, `ui/modules/import_cab.dart`):

- **Registro CAB:** `CAB\0` + nome (14 B) + `01` + "CAB Level" (padrão 50) +
  **2048 amostras int24 LE mono a 44,1 kHz**, 6 164 B no total. IRs maiores
  são truncadas em cerca de 46 ms.
- **Reamostragem** (`WaveUtils.resample`): sinc com janela Kaiser β=8,6,
  ±24 taps, corte `min(1, dst/src)`, normalizada pela soma dos pesos,
  arredondada e limitada a int24. Reproduzir exatamente esse algoritmo
  garante saída idêntica à do editor.
- **WAV de entrada:** o editor falha com Float32 ("device busy"). Converter
  para PCM inteiro antes, o que o nosso app faz sozinho.
- **Sequência no MK-300:** blob em pedaços de cerca de 1015 B, com ACK a
  cada pedaço → *audition* (`0xA` = 2 para CAB) → *save* (`0xF`, slot) e
  espera do commit na flash (alguns segundos; comandos enviados durante o
  commit podem cancelá-lo). O IR Box usa pedaços de 173 B. ❓ No Tank-G,
  capturar tamanho de pedaço, endereço e comando de save.
- **Amp (`.am2data`):** registro `AM2\0` de até 16 KiB, provavelmente gravado
  pelo mesmo caminho do CAB. ❓

---

## 7. Armadilhas conhecidas (de quem já passou por isso)

1. **Buffer velho:** leitura fora do mapa devolve uma resposta bem formada,
   com checksum válido e lixo dentro. Validar o tamanho esperado e conferir o
   conteúdo; resposta válida não prova leitura válida.
2. **`0x21` ERASE e as características BLE de OTA (`ae00`–`ae02`): nunca.**
3. Nunca escrever fora do mapa conhecido. O pedal não rejeita.
4. Commit na flash leva segundos. Não mandar nada até confirmar por leitura.
5. Escrever um modelo reseta os knobs daquele bloco para o padrão do modelo.
   Reler depois de escrever.
6. Em alguns pedais o display só relê os nomes no boot.
7. **Estado emprestado:** o Loopback (e qualquer modo de áudio USB) deve
   voltar ao estado anterior em qualquer saída do programa, inclusive em
   SIGINT. No MK-300, alguém ficou uma hora com o rig mudo por causa disso.
8. Editor oficial e app novo **não podem ser usados ao mesmo tempo.** O
   M-EFCS guarda o próprio cache do preset e sobrescreve as mudanças.
9. Antes de qualquer escrita na flash: **backup completo**, e exigir
   confirmação explícita.

---

## 8. Referências

Pedais da mesma família, todos falando com o M-EFCS:

- **Tank Mini**, protocolo completo (PT-BR): https://github.com/xxyurirangelxx/mvave-tank-mini-protocol
- **MK-300**, SysEx + upload NAM/IR byte-exato, decompilação do M-EFCS com blutter: https://github.com/jpfaria/mvave (`docs/protocol.md`, `docs/learnings.md`)
- **BlackBox no Linux**, controle nativo, `irload.py`, patch do M-EFCS para Wine: https://github.com/Bulrok/mvave-blackbox-linux
- **BlackBox RE**, mapa de famílias DEV, notas do M-EFCS: https://github.com/jakino0/mvave-blackbox-reverse-engineering (`docs/mefcs-protocol.md`)
- **IR Box**, frame, checksum, empacotamento, upload de IR e armadilhas: https://github.com/marco-jardim/mvave-irbox-tools (`docs/usb.md`, `docs/protocol.md`)
- **Cube Baby**, crate **Rust** sobre USB-MIDI: https://github.com/pferreir/cuvave-midi
- Índice da comunidade M-VAVE: https://github.com/cicloid/awesome-fm-1 (seção "Related M-VAVE projects")
- Filtro Wireshark para SysEx: `usbaudio.sysex.reassembled.data` (wiki do Mixxx: https://github.com/mixxxdj/mixxx/wiki/Reverse-Engineering)
