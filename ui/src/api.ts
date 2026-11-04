// Ponte com o backend Rust. Fora do Tauri (vite no navegador) usa um pedal
// simulado, para dar para mexer na interface sem a Tank-G.

import { invoke } from "@tauri-apps/api/core";

export type Mode = "P" | "L";

export interface DeviceInfo {
  firmware: string;
  port: string;
}

export interface DeviceStatus {
  /** "2B", ou null se nenhum preset foi escolhido desde que a pedaleira ligou. */
  preset: string | null;
  mode: Mode;
}

/** Preset atual (21 bytes decodificados). Knobs 0–127. */
export interface Patch {
  ampOn: boolean;
  modOn: boolean;
  delayOn: boolean;
  reverbOn: boolean;
  noiseGate: number;
  /** 0–8 */
  ampType: number;
  gain: number;
  treble: number;
  middle: number;
  bass: number;
  volume: number;
  modFx: number;
  modSpeed: number;
  delayMix: number;
  delayTime: number;
  reverbDecay: number;
  reverbMix: number;
  /** 0 = OFF, 1–8 = slot */
  irCab: number;
  unknown: [number, number, number];
}

export interface DeviceState {
  status: DeviceStatus;
  patch: Patch;
}

export const LETTERS = ["A", "B", "C", "D"];
export const BANKS = 9;
export const PRESET_IDS = Array.from({ length: BANKS * 4 }, (_, i) => `${Math.floor(i / 4) + 1}${LETTERS[i % 4]}`);

export const inTauri = "__TAURI_INTERNALS__" in window;

export function programChangeHex(id: string): string {
  return `C0 ${PRESET_IDS.indexOf(id).toString(16).padStart(2, "0").toUpperCase()}`;
}

const mock = {
  status: { preset: "4C", mode: "P" } as DeviceStatus,
  // 4C de fábrica
  patch: {
    ampOn: true, modOn: false, delayOn: true, reverbOn: true, noiseGate: 0, ampType: 5,
    gain: 85, treble: 88, middle: 64, bass: 61, volume: 98, modFx: 0, modSpeed: 0,
    delayMix: 6, delayTime: 35, reverbDecay: 5, reverbMix: 1, irCab: 4, unknown: [0, 0, 0],
  } as Patch,
};

export async function connect(): Promise<DeviceInfo> {
  if (!inTauri) return { firmware: "TANK-Gv2_097 (simulado)", port: "simulado" };
  return invoke<DeviceInfo>("device_info");
}

export async function readStatus(): Promise<DeviceStatus> {
  if (!inTauri) return { ...mock.status };
  return invoke<DeviceStatus>("device_status");
}

/** Estado + preset atual numa leitura só (o que o app acompanha). */
export async function readState(): Promise<DeviceState> {
  if (!inTauri) return { status: { ...mock.status }, patch: { ...mock.patch } };
  return invoke<DeviceState>("device_state");
}

export async function selectPreset(id: string): Promise<DeviceStatus> {
  if (!inTauri) {
    mock.status.preset = id;
    return { ...mock.status };
  }
  return invoke<DeviceStatus>("select_preset", { id });
}

/** Salva o banco de presets em `path`. Devolve o número de bytes. */
export async function backup(path: string): Promise<number> {
  if (!inTauri) return 758;
  return invoke<number>("backup", { path });
}

export async function pickBackupPath(): Promise<string | null> {
  if (!inTauri) return "/tmp/tankg-simulado.bin";
  const { save } = await import("@tauri-apps/plugin-dialog");
  const date = new Date().toISOString().slice(0, 10);
  return save({
    title: "Salvar backup dos presets",
    defaultPath: `tankg-presets-${date}.bin`,
    filters: [{ name: "Banco de presets Tank-G", extensions: ["bin"] }],
  });
}
