# Tank-GL

Editor nativo para Linux da pedaleira **M-VAVE Tank-G**, para substituir o
M-EFCS (só Windows). Tauri 2 + Rust + React.

> Fase atual: **engenharia reversa + CLI só de leitura.** O pedal já responde
> (`TANK-Gv2_097`); ver `docs/PROTOCOLO.md` seção 0.

## CLI

```bash
cargo build --release
target/release/tankg identify          # TANK-Gv2_097
target/release/tankg status            # preset 2A  modo P (preset)
target/release/tankg preset 3C         # ou 1–36, next, prev
target/release/tankg backup banco.bin  # banco de presets (758 B)
target/release/tankg raw 0x20000000 4  # leitura crua, debug
```

Escrita na flash (`0x22`) e ERASE (`0x21`) estão bloqueados no transporte.

## Documentos

| Doc | Conteúdo |
|---|---|
| [docs/PLANO.md](docs/PLANO.md) | O que o app oficial faz e o que o nosso adiciona |
| [docs/ARQUITETURA.md](docs/ARQUITETURA.md) | Stack, módulos, como cada recurso funciona, roteiro por fases |
| [docs/PROTOCOLO.md](docs/PROTOCOLO.md) | Como falar com o Tank-G: USB-MIDI SysEx, frame `00 59`, mapa de memória, upload de IR, armadilhas |
| [docs/CAPTURAS.md](docs/CAPTURAS.md) | M-EFCS no Wine, captura com usbmon, checklist |
| [design/README.md](design/README.md) | Design system **Tank Studio**: cores, tipografia, tokens, 22 componentes React e ícones (cada componente tem `README.md` + `preview.html`) |

## Resumo

- O Tank-G é USB-MIDI class-compliant com SysEx, mais interface de áudio USB.
  No Linux funciona sem driver.
- Usa o protocolo comum dos pedais M-VAVE, que a comunidade já documentou no
  Tank Mini, MK-300, BlackBox e IR Box.
- O pedal tem 36 presets, **9 slots de amp e 8 de IR**. O app trata esses
  slots como cache de uma biblioteca ilimitada no PC.
