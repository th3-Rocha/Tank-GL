# PresetEditor

A tela de edição completa de um preset: display de LED, nome, cor do LED, status de conexão, "Gravar no pedal", a `SignalChain` e os knobs do módulo escolhido.

- Props: `preset` (valores iniciais), `presetId`, `presets` (para o seletor), `ampTypes`, `irNames`, `irSlots`, `connection`, `onChange(preset, path, value)`, `onSave(preset, id)`.
- Mexer em qualquer valor marca "Modificado" e habilita Gravar e Reverter; duplo clique num knob volta ao valor salvo.
- O módulo CAB/IR mostra o slot (knob), o nome da IR (select) e o IR LV (fader).
