# Arquitetura e stack

## 1. Visão geral

```
┌──────────────── Tank-GL (Tauri 2) ────────────────┐      ┌──────────────┐
│  UI web (React + TS)                              │      │ tankg (CLI)  │
│  tabela de presets · cadeia · biblioteca · Ctrl+K │      │ binds Hypr   │
└──────────────┬────────────────────────────────────┘      └──────┬───────┘
               │ invoke / events (Tauri IPC)                      │ socket Unix
┌──────────────▼──────────────────────────────────────────────────▼───────┐
│  Backend Rust (processo do app = dono único da porta MIDI)              │
│                                                                         │
│  tankg-engine   estado, sync, cache de slots, setlists, undo, backup    │
│  tankg-library  índice SQLite, scan, hash, tags, conversão de IR        │
│  tankg-device   transporte MIDI, req/resp, ACK, upload em pedaços       │
│  tankg-proto    puro, sem I/O: frame, pack7, checksum, structs, blobs   │
│  tankg-audio    pré-escuta: DI + convolução da IR → PipeWire            │
└──────────────┬──────────────────────────────────────────────────────────┘
               │ ALSA sequencer (midir)          │ áudio (cpal/PipeWire)
          ┌────▼────────────── Tank-G (USB) ─────▼────┐
          │  USB-MIDI SysEx            USB Audio 2x2  │
          └───────────────────────────────────────────┘
```

Princípios:
- **`tankg-proto` é puro.** Só bytes entrando e saindo, sem I/O. Testado com
  os vetores dos outros pedais e, depois, com as nossas capturas. É a parte
  mais crítica e a mais fácil de testar.
- **Só um processo fala com o pedal por vez.** Quando o app está aberto, ele é
  o dono da porta e a CLI conversa com ele por socket. Com o app fechado, a
  CLI abre o pedal direto (mesma crate `tankg-device`).
- **O PC é a fonte da verdade.** O pedal guarda uma cópia: 36 presets, 9 amps
  e 8 IRs. Biblioteca, histórico e setlists ficam no PC.
- **Escrita na flash é sempre explícita.** Antes, backup automático. Comandos
  perigosos (`0x21` ERASE) ficam bloqueados em uma denylist na camada de
  transporte.

## 2. Stack

### Shell e front

| Peça | Escolha | Motivo |
|---|---|---|
| App | **Tauri 2** | O pedido. Binário pequeno, backend Rust, WebKitGTK no Linux (já instalado: webkit2gtk-4.1 2.52) |
| UI | **React 19 + TypeScript + Vite** | Ecossistema com as duas peças-chave abaixo |
| Tabela densa | **TanStack Table + TanStack Virtual** | 36 presets × cerca de 25 parâmetros, ordenação, seleção e edição de várias linhas |
| Paleta Ctrl+K | **cmdk** | É exatamente esse componente |
| Estado | **Zustand** + store derivado dos eventos do backend | Simples. O backend manda `state-changed` e a UI só reflete |
| Undo/redo na UI | Zundo (middleware do Zustand) | O histórico persistente de verdade fica no backend |
| Visual | **Design system em `design/`** (Tank Studio) | `tokens.json` é a fonte de verdade → `tokens.css` (variáveis CSS). Os 22 componentes de `design/components/` (Knob, SignalChain, PresetGrid, CommandPalette, LibraryTable, MidiMonitor…) são a referência; serão portados para módulos TSX em `ui/` |
| Fontes | Barlow, Barlow Condensed, JetBrains Mono | Empacotadas localmente (ex.: `@fontsource`) porque o app precisa funcionar offline, sem Google Fonts |
| Estilo | CSS com as variáveis de `tokens.css` (Tailwind opcional, mapeando os tokens no tema) | Um único tema: Tank-G |
| Pacotes | pnpm (11.3 instalado) | |

### Backend Rust

| Função | Crate | Observação |
|---|---|---|
| MIDI | **`midir`** (ALSA seq) | Convive com o PipeWire. Habilitar SysEx (`Ignore::None`) |
| Achar o pedal | `udev` / sysfs | Casar VID:PID e não só o nome "Composite"; hotplug |
| Async | `tokio` | Fila de comandos com timeout, ACK e retry |
| Serialização | `serde`, `serde_json` | Presets e setlists em JSON legível |
| Biblioteca | **`rusqlite`** (+ FTS5) | Índice, tags, favoritos, histórico, mapa slot→hash |
| Busca fuzzy | **`nucleo`** | O matcher do Helix: rápido e com boa ordenação |
| Hash/dedup | `blake3` | Identidade de IR/amp = hash do conteúdo |
| Vigiar pastas | `notify` | Pasta de IRs atualiza sozinha |
| WAV | `hound` | Leitura PCM e float de qualquer formato |
| Reamostragem | **implementação própria** do sinc Kaiser do M-EFCS | Saída byte a byte idêntica à do editor. `rubato` só para a pré-escuta |
| Pré-escuta | `cpal` + `realfft`/`fft-convolver` + `symphonia` (DI em mp3/flac) | Convolução local, sem gravar na flash |
| CLI | `clap` | `tankg …` |
| IPC CLI↔app | socket Unix em `$XDG_RUNTIME_DIR/tankg.sock`, JSON por linha | |
| Plugins Tauri | `dialog`, `fs`, `single-instance`, `global-shortcut` (opcional) | |
| Erros e log | `thiserror`, `anyhow`, `tracing` | Log com o hex do SysEx (TX/RX), útil como captura |

## 3. Estrutura do repositório

```
Tank-GL/
├─ Cargo.toml                 workspace
├─ crates/
│  ├─ tankg-proto/            frame, pack7, checksum, Command/Event, Patch(92B), CabRecord, AmpRecord
│  ├─ tankg-device/           midir, fila req/resp, ACK, upload, denylist, modo "trace" (JSONL)
│  ├─ tankg-library/          SQLite, scan, conversão de IR, metadados
│  ├─ tankg-engine/           estado, sync PC↔pedal, cache de slots, setlists, histórico, backup
│  ├─ tankg-audio/            pré-escuta
│  └─ tankg-cli/              binário `tankg`
├─ src-tauri/                 app Tauri: comandos e eventos sobre o engine
├─ ui/                        React
├─ design/                    design system Tank Studio (tokens, componentes, ícones)
├─ docs/                      PLANO, ARQUITETURA, PROTOCOLO, CAPTURAS
└─ captures/                  pcaps locais (fora do git) + README com metadados
```

## 4. Como cada recurso funciona

### 4.1 Conexão e estado
1. O udev detecta o Tank-G e o app abre a porta MIDI.
2. Handshake: consulta `11` (modelo/firmware) e escolha do perfil pelo
   firmware (am3/am4).
3. Leitura do banco de presets (família 0) e do patch atual (família 1), e
   montagem do `DeviceState`.
4. Daí em diante, cada mudança da UI vira **escrita no patch atual (RAM)**,
   com resposta instantânea. "Salvar" grava o preset na flash.
5. Mudanças feitas no pedal (footswitch): polling leve do estado
   (`0x2000_0000`?) ou notificação, a confirmar.

### 4.2 Slots como cache (IR e amp)
- O preset no PC guarda `ir_ref = blake3` e `amp_ref = blake3`, e não números
  de slot.
- Uma tabela `slot_map(kind, slot, hash, last_used, pinned)` espelha o pedal.
  Ela é validada pelo nome e pelo conteúdo lido do slot, porque alguém pode
  ter usado o editor oficial no meio.
- Ao carregar o preset: se a IR já está no pedal, só aponta o slot. Se não
  está, grava no **slot LRU não fixado**, reescreve o `ir_slot` do preset e só
  então seleciona.
- **Modo show:** escolhida a setlist, o app calcula as IRs e amps
  necessárias. Se couberem (≤ 8 IRs e ≤ 9 amps), grava tudo antes e **fixa**
  os slots. Ao vivo, nenhuma gravação acontece. Se não couberem, o app avisa
  antes do show.

### 4.3 Biblioteca
- Arrastar uma pasta → scan recursivo de `.wav` e `.am2data` → hash →
  SQLite. Duplicadas são agrupadas pelo hash.
- Metadados: tags tiradas do nome do arquivo (falante, mic, tamanho de caixa
  como "412" ou "112") mais tags manuais e favoritos.
- **Conversão de IR:** qualquer WAV (int/float, mono/estéreo, qualquer taxa)
  → mono → 44,1 kHz (sinc Kaiser) → 2048 amostras int24 → registro `CAB\0`.
  O app avisa quanto do rabo da IR foi cortado.

### 4.4 Presets densos
- Tabela 36 × parâmetros; edição de várias linhas aplica o valor em N presets
  e grava em lote com uma confirmação.
- Biblioteca ilimitada de presets no PC. "Sincronizar" envia 36 presets para o
  pedal.
- Diff lado a lado: comparação estrutural dos dois `Preset`.
- Histórico: cada gravação de preset vira uma versão no SQLite. Undo/redo da
  sessão no frontend.
- Backup completo: banco de presets + 17 registros AMP/CAB, num arquivo
  `.tankgl` (zip com JSON + blobs). Restauração exige confirmação.

### 4.5 CLI (Hyprland)
```
tankg preset 5B            # ou tankg preset 14
tankg preset next|prev
tankg bypass toggle
tankg ir load ~/irs/v30.wav [--slot 3] [--yes]
tankg ir list | tankg amp list
tankg backup ~/tankg-$(date +%F).tankgl
tankg monitor              # monitor MIDI com decode do frame
tankg raw read 04 0x10000000 0x5C     # debug, só leitura
```
Exemplo de bind: `bind = SUPER, F5, exec, tankg preset 1A`.

### 4.6 Monitor MIDI
Mostra TX/RX com hex do fio, frame desempacotado e decode (cmd, tipo,
endereço, dados, checksum ok). Exporta JSONL. O mesmo formato serve de
fixture de teste.

## 5. Roteiro

| Fase | Entrega | Depende de |
|---|---|---|
| **0. Captura** | Wine + usbmon, checklist do `CAPTURAS.md`, decodes em `PROTOCOLO.md` | ter o pedal |
| **1. Proto + CLI só de leitura** | `tankg-proto` com testes; `tankg identify`, `dump`, `monitor`, `backup` | fase 0 (handshake/leitura) |
| **2. Controle ao vivo** | trocar preset, knobs, toggles, bypass, loopback pela CLI | captura dos knobs |
| **3. App Tauri básico** | cadeia de efeitos, lista de presets, salvar, cor, swap, export/import | fase 2 |
| **4. IR/amp** | conversão + upload + restaurar fábrica | captura do upload |
| **5. Biblioteca + cache de slots** | scan, busca, tags, LRU, modo show | fase 4 |
| **6. Extras** | tabela densa, Ctrl+K, setlists, diff, histórico, pré-escuta | fase 5 |

As fases 1 e 2 já dão para testar com os comandos do Tank Mini (sec. 5.1 do
`PROTOCOLO.md`). Se o Tank-G responder igual, ganhamos semanas.
