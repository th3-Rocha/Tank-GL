# Capturas: como fazer e o que capturar

Objetivo: confirmar no Tank-G real as hipóteses de `PROTOCOLO.md`. São duas
frentes que se complementam: **captura dinâmica** do tráfego USB e **análise
estática** do código Dart do editor.

## 1. Rodar o M-EFCS no Linux (sem VM)

O editor de Windows roda no Wine. O único problema é o plugin de Bluetooth,
que aborta na inicialização porque o Wine não implementa
`Windows.Devices.Radios`. A correção é trocar o primeiro byte de
`universal_ble_plugin.dll` (único export, o registrar do plugin) por `C3`
(`ret`), o que desliga o plugin. Fonte e script: Bulrok/mvave-blackbox-linux
(`tools/patch_mefcs.py`).

```bash
export WINEPREFIX=~/.local/share/wineprefixes/mvave
wineboot -i
winetricks dxvk                                   # opcional
unzip ~/Downloads/M-EFCS.zip -d "$WINEPREFIX/drive_c/"
# Patch: offset 0xBB0 do universal_ble_plugin.dll, 40 53 48 83 → C3 53 48 83
# O script do Bulrok confere o hash da build dele. A nossa (2026-10-10) pode
# ser outra, então confira os bytes no offset antes de gravar. Guarde o .orig.
wine "$WINEPREFIX/drive_c/M-EFCS/m_efcs.exe"
```

Se o Wine não funcionar: VM Windows com USB passthrough. O resto do método
continua igual, porque o usbmon roda no host.

## 2. Captura dinâmica com usbmon

```bash
lsusb                                  # anotar VID:PID e o barramento do Tank-G
aconnect -l                            # anotar o nome da porta MIDI
sudo modprobe usbmon
sudo tshark -i usbmon<BUS> -w captures/<acao>.pcapng
# análise: Wireshark com o filtro  usbaudio.sysex.reassembled.data
```

Também dá pra ver só o que o pedal envia com `aseqdump -p <porta>`. Para o
sentido host→pedal, use usbmon.

Regras:
- **Uma ação por arquivo.** Nome = ação (`knob-reverb-mix-0-50-100.pcapng`).
- Esperar uns 5 s parado antes de cada ação, para separar o polling do editor
  da ação em si.
- Anotar o firmware (consulta `11`) e a versão do M-EFCS em
  `captures/README.md`.
- Os `.pcapng` ficam fora do git (`.gitignore`). Os decodes e notas vão para
  `docs/`.

## 3. Checklist (do plano, com o que cada item responde)

- [ ] **Abrir o editor com o pedal conectado.** Mostra o handshake, a consulta
      de identidade e os dumps iniciais. É o mapa de memória de graça.
- [ ] **Girar cada knob em 0 → meio → máx.** Mostra endereço do knob, largura
      (u8/u16) e escala.
- [ ] **Ligar e desligar cada bloco e o Bypass.** Mostra os endereços de
      toggle e se o bypass é global ou por preset.
- [ ] **Trocar de preset, mudar a cor, Save preset to, Swap.** Mostra o
      select (`0xE…`?), onde fica a cor, o save na flash (`id·92`?) e se o
      swap é um comando próprio ou dois saves.
- [ ] **Export current preset e Export all presets.** Mostra o formato do
      arquivo exportado e o dump do banco (família 0?).
- [ ] **Gravar uma IR num slot.** É a mais importante: tamanho de pedaço,
      endereço, audition e save. Comparar o blob com o registro `CAB\0` gerado
      pelo `irload.py` com a mesma WAV.
- [ ] **Gravar um AM2Data.** Mostra se usa o mesmo caminho do CAB e o tamanho
      do registro `AM2\0`.
- [ ] **Ligar e desligar o Loopback.** Confirma `0xD000_0000`.
- [ ] **IR Lv.** Mostra se é global ou por preset, e o endereço.
- [ ] **Factory reset (presets, amps/IRs).** Fazer **por último**, e só depois
      de um backup. Mostra se o editor reenvia os `.bin` de fábrica embutidos
      ou se é um comando do pedal.
- [ ] **Trocar de preset pelo footswitch** (sem mexer no editor). Mostra se o
      pedal avisa o host ou se o editor faz polling.

## 4. Análise estática (em paralelo)

- **`data/app.so` do Windows** é Dart AOT x86-64. Dá pra extrair strings e
  nomes de classe (assim achamos `tank_g/structs.dart`). Recuperar código é
  mais difícil.
- **APK Android do M-EFCS + [blutter](https://github.com/worawit/blutter)**
  recupera o código Dart (arm64): `tank_g/structs.dart`, `effect_params.dart`,
  `sysex_codec.dart`, `wave_utils.dart`. Foi o caminho que o jpfaria/mvave
  usou para fechar o upload byte a byte. A Dart 3.11.x é suportada.
- **`resources/bin/tankG/*.bin`** traz os bancos de fábrica. Servem de
  referência para os formatos AMP/CAB e como "restaurar fábrica" offline.
  São do fabricante: **não versionar**, ler em tempo de execução de um
  M-EFCS que o usuário tenha.

## 5. O que precisamos do pedal (primeiro plug)

Somente leitura, nada de escrita:

1. `lsusb -v -d <vid:pid>` e `aconnect -l`
2. Consulta `11` (identidade/firmware)
3. Leitura do patch atual (`04 / 0x1000_0000 / 0x5C`) e conferência contra o
   que aparece no display
4. Leitura do banco (`04 / 0x0000_0000`) em pedaços de até 1000 B, para
   backup e validação do tamanho
