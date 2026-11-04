import { useState } from "react";
import type { ChainNode } from "../../design/components/index.d.ts";
import type { Patch } from "./api";

const T = window.Tank;
const MAX = 127;

// Nomes dos amps de fábrica (am4, `tankG_am4_ampCab.bin`). Se alguém trocar o
// amp de um slot pelo editor oficial, o nome aqui fica errado: ainda não lemos
// os nomes do pedal.
const AMP_NAMES = ["J120 CL", "J900 CL", "VAVE TS", "RAT OD", "VAVE HTS", "VAVE DS", "MS GV2", "J900 HV", "MS HIGAIN"];
const IR_LABELS = ["OFF", "1", "2", "3", "4", "5", "6", "7", "8"];
const HINT = "Só leitura por enquanto: gire o knob na pedaleira, no modo LIVE";

type Block = "gate" | "amp" | "mod" | "delay" | "reverb" | "cab";

interface Props {
  patch: Patch;
  live: boolean;
}

export function PatchView({ patch: p, live }: Props) {
  const [selected, setSelected] = useState<Block>("amp");

  const nodes: ChainNode[] = [
    { id: "gate", kind: "gate", label: "GATE", on: p.noiseGate > 0, led: "green", sub: p.noiseGate > 0 ? `Gate ${p.noiseGate}` : "Aberto", amount: Math.round((p.noiseGate / MAX) * 100) },
    { id: "amp", kind: "amp", label: "AMP", on: p.ampOn, sub: AMP_NAMES[p.ampType] ?? `Tipo ${p.ampType + 1}` },
    { id: "mod", kind: "stomp", label: "MOD", on: p.modOn, led: "purple", sub: `FX ${p.modFx}` },
    { id: "delay", kind: "stomp", label: "DELAY", on: p.delayOn, led: "cyan", sub: `Time ${p.delayTime}` },
    { id: "reverb", kind: "stomp", label: "REVERB", on: p.reverbOn, led: "blue", sub: `Decay ${p.reverbDecay}` },
    { id: "cab", kind: "cab", label: "CAB · IR", on: p.irCab > 0, sub: p.irCab > 0 ? `Slot ${p.irCab}` : "OFF" },
  ];

  const knob = (label: string, value: number, size: "sm" | "md" | "lg" = "md", extra: object = {}) => (
    <T.Knob key={label} label={label} value={value} min={0} max={MAX} size={size} hint={HINT} {...extra} />
  );

  const block = (id: Block, el: React.ReactElement) => (
    <div key={id} className={`patch__block ${selected === id ? "is-selected" : ""}`} onClick={() => setSelected(id)}>
      {el}
    </div>
  );

  return (
    <div className="patch">
      <T.SignalChain nodes={nodes} selected={selected} onSelect={(id) => setSelected(id as Block)} signal={p.ampOn} label="Cadeia de sinal" />
      <div className="patch__grid">
        {block("gate", <T.EffectBlock title="NOISE GATE" subtitle="antes do amp">{knob("NOISE GATE", p.noiseGate)}</T.EffectBlock>)}
        {block(
          "amp",
          <T.EffectBlock title="AMP" subtitle={AMP_NAMES[p.ampType]} enabled={p.ampOn}>
            {knob("TYPE", p.ampType, "md", { min: 0, max: 8, valueLabels: AMP_NAMES })}
            {knob("GAIN", p.gain, "lg")}
            {knob("TREBLE", p.treble)}
            {knob("MIDDLE", p.middle)}
            {knob("BASS", p.bass)}
            {knob("VOLUME", p.volume, "lg")}
          </T.EffectBlock>,
        )}
        {block(
          "mod",
          <T.EffectBlock title="MOD" enabled={p.modOn}>
            {knob("MOD FX", p.modFx)}
            {knob("MOD SPEED", p.modSpeed)}
          </T.EffectBlock>,
        )}
        {block(
          "delay",
          <T.EffectBlock title="DELAY" enabled={p.delayOn}>
            {knob("DLY MIX", p.delayMix)}
            {knob("DLY TIME", p.delayTime)}
          </T.EffectBlock>,
        )}
        {block(
          "reverb",
          <T.EffectBlock title="REVERB" enabled={p.reverbOn}>
            {knob("RVB DECAY", p.reverbDecay)}
            {knob("RVB MIX", p.reverbMix)}
          </T.EffectBlock>,
        )}
        {block(
          "cab",
          <T.EffectBlock title="CAB · IR" subtitle={p.irCab > 0 ? `slot ${p.irCab}` : "desligado"}>
            {knob("IR CAB", p.irCab, "md", { min: 0, max: 8, valueLabels: IR_LABELS, ticks: true })}
          </T.EffectBlock>,
        )}
      </div>
      <p className="hint">
        {live
          ? "Modo LIVE: o que você gira e liga na pedaleira aparece aqui na hora. Nada foi salvo até você salvar na pedaleira."
          : "Modo PRESET: os knobs da pedaleira não editam. Pise em B + C juntos para o modo LIVE."}{" "}
        Nomes dos amps são os de fábrica. O MASTER é global e não fica no preset.
      </p>
    </div>
  );
}
