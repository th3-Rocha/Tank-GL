import type * as React from 'react';

export type LedColor = 'red' | 'orange' | 'yellow' | 'green' | 'cyan' | 'blue' | 'purple' | 'white';
export type Option<V = string | number> = V | { value: V; label: string; icon?: string; disabled?: boolean };

export interface IconProps { name: string; size?: number; title?: string; className?: string; strokeWidth?: number }
export declare function Icon(props: IconProps): React.ReactElement | null;

export interface KbdProps { children: string; className?: string }
export declare function Kbd(props: KbdProps): React.ReactElement;

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'secondary' | 'primary' | 'ghost' | 'danger' | 'color';
  size?: 'sm' | 'md' | 'lg';
  icon?: string; iconRight?: string; kbd?: string; block?: boolean; loading?: boolean;
}
export declare function Button(props: ButtonProps): React.ReactElement;

export interface SegmentedControlProps<V = string> { options: Option<V>[]; value?: V; defaultValue?: V; onChange?: (v: V) => void; label?: string; size?: 'sm' | 'md'; className?: string }
export declare function SegmentedControl<V = string>(props: SegmentedControlProps<V>): React.ReactElement;

export interface KnobProps {
  label?: string; value?: number; defaultValue?: number; onChange?: (v: number) => void;
  min?: number; max?: number; step?: number; size?: 'sm' | 'md' | 'lg'; unit?: string;
  format?: (v: number) => string; valueLabels?: string[]; ticks?: boolean; resetValue?: number;
  disabled?: boolean; hint?: string; className?: string;
}
export declare function Knob(props: KnobProps): React.ReactElement;

export interface FaderProps {
  label?: string; ariaLabel?: string; value?: number; defaultValue?: number; onChange?: (v: number) => void;
  min?: number; max?: number; step?: number; unit?: string; format?: (v: number) => string;
  width?: number | string; resetValue?: number; showValue?: boolean; disabled?: boolean; className?: string;
}
export declare function Fader(props: FaderProps): React.ReactElement;

export interface ToggleProps { label?: string; checked?: boolean; defaultChecked?: boolean; onChange?: (on: boolean) => void; size?: 'sm' | 'md'; hideLabel?: boolean; showState?: boolean; onText?: string; offText?: string; disabled?: boolean; className?: string }
export declare function Toggle(props: ToggleProps): React.ReactElement;

export interface SelectProps<V = string | number> { label?: string; hideLabel?: boolean; options: Option<V>[]; value?: V; defaultValue?: V; onChange?: (v: V) => void; size?: 'sm' | 'md'; width?: number | string; disabled?: boolean; className?: string }
export declare function Select<V = string | number>(props: SelectProps<V>): React.ReactElement;

export interface LedColorPickerProps { value?: LedColor; defaultValue?: LedColor; onChange?: (c: LedColor) => void; colors?: { value: LedColor; label: string }[]; showName?: boolean; label?: string; className?: string }
export declare function LedColorPicker(props: LedColorPickerProps): React.ReactElement;

export interface LedDisplayProps { value: string; digits?: number; height?: number; color?: string; caption?: string; label?: string; className?: string }
export declare function LedDisplay(props: LedDisplayProps): React.ReactElement;

export interface ConnectionStatusProps { transport?: 'usb' | 'ble'; state?: 'connected' | 'connecting' | 'disconnected' | 'error'; device?: string; detail?: string; onClick?: () => void; className?: string }
export declare function ConnectionStatus(props: ConnectionStatusProps): React.ReactElement;

export interface TagProps { tone?: 'neutral' | 'amber' | 'ok' | 'warn' | 'danger'; size?: 'sm' | 'md'; icon?: string; count?: number; selected?: boolean; onClick?: () => void; onRemove?: () => void; children: React.ReactNode; className?: string }
export declare function Tag(props: TagProps): React.ReactElement;

export interface ProgressBarProps { label?: string; value?: number; detail?: string; tone?: 'amber' | 'ok' | 'danger'; className?: string }
export declare function ProgressBar(props: ProgressBarProps): React.ReactElement;

export interface SearchFieldProps { value?: string; defaultValue?: string; onChange?: (q: string) => void; placeholder?: string; label?: string; width?: number | string; size?: 'md' | 'lg'; shortcut?: string | null; hotkey?: boolean; autoFocus?: boolean; onKeyDown?: React.KeyboardEventHandler<HTMLInputElement>; className?: string }
export declare function SearchField(props: SearchFieldProps): React.ReactElement;

export interface CommandItem { id: string; kind: 'action' | 'preset' | 'ir' | 'amp' | 'param' | 'setlist' | string; title: string; subtitle?: string; meta?: string; icon?: string; color?: LedColor; keywords?: string }
export interface CommandPaletteProps { items: CommandItem[]; onSelect?: (item: CommandItem) => void; onClose?: () => void; open?: boolean; inline?: boolean; defaultQuery?: string; placeholder?: string; label?: string; limitPerGroup?: number; maxHeight?: number | string; autoFocus?: boolean; className?: string }
export declare function CommandPalette(props: CommandPaletteProps): React.ReactElement | null;

export interface LibraryItem { id: string; name: string; file?: string; tags?: string[]; format?: string; slot?: number | null; favorite?: boolean }
export interface LibraryTableProps { items: LibraryItem[]; query?: string; tags?: string[]; selected?: string[]; onSelectionChange?: (ids: string[]) => void; onPreview?: (id: string) => void; onSend?: (id: string) => void; onToggleFavorite?: (id: string) => void; playingId?: string | null; maxHeight?: number | string; noun?: string; label?: string; footer?: boolean; footerActions?: React.ReactNode; defaultSort?: { key: 'name' | 'slot' | 'fav'; dir: 1 | -1 }; className?: string }
export declare function LibraryTable(props: LibraryTableProps): React.ReactElement;

export interface PresetSummary { id: string; name: string; color?: LedColor | null; modified?: boolean; ir?: string }
export interface PresetGridProps { presets: PresetSummary[]; value?: string; defaultValue?: string; onSelect?: (id: string) => void; onActivate?: (id: string) => void; activeId?: string; query?: string; banks?: number; letters?: string[]; label?: string; className?: string }
export declare function PresetGrid(props: PresetGridProps): React.ReactElement;

export interface PanelProps { title?: React.ReactNode; icon?: string; meta?: React.ReactNode; actions?: React.ReactNode; flush?: boolean; sunken?: boolean; style?: React.CSSProperties; className?: string; children?: React.ReactNode }
export declare function Panel(props: PanelProps): React.ReactElement;

export interface EffectBlockProps { title: string; subtitle?: string; enabled?: boolean; defaultEnabled?: boolean; onToggle?: (on: boolean) => void; style?: React.CSSProperties; className?: string; children?: React.ReactNode }
export declare function EffectBlock(props: EffectBlockProps): React.ReactElement;

export interface ChainNode { id: string; kind: 'gate' | 'amp' | 'stomp' | 'cab'; label: string; on: boolean; sub?: string; led?: LedColor; amount?: number }
export interface SignalChainProps { nodes?: ChainNode[]; selected?: string; defaultSelected?: string; onSelect?: (id: string) => void; onToggle?: (id: string, on: boolean) => void; signal?: boolean; label?: string; className?: string }
export declare function SignalChain(props: SignalChainProps): React.ReactElement;

export interface TankPreset {
  name: string; color: LedColor; ircab: number; irlv: number; gate: number;
  reverb: { on: boolean; mix: number; decay: number }; delay: { on: boolean; time: number; mix: number };
  mod: { on: boolean; speed: number; fx: number };
  amp: { on: boolean; volume: number; bass: number; middle: number; treble: number; gain: number; type: number };
}
export interface PresetEditorProps { preset?: TankPreset; presetId?: string; presets?: PresetSummary[]; ampTypes?: string[]; irNames?: string[]; irSlots?: number; connection?: ConnectionStatusProps | false; defaultSelected?: string; onChange?: (preset: TankPreset, path: string, value: unknown) => void; onSave?: (preset: TankPreset, id: string) => void; className?: string }
export declare function PresetEditor(props: PresetEditorProps): React.ReactElement;

export interface MidiMessage { time: string; dir: 'in' | 'out'; bytes: string; text?: string; highlight?: boolean }
export interface MidiMonitorProps { messages: MidiMessage[]; paused?: boolean; onTogglePause?: () => void; onClear?: () => void; height?: number; title?: string; className?: string }
export declare function MidiMonitor(props: MidiMonitorProps): React.ReactElement;

declare global {
  interface Window {
    Tank: {
      Icon: typeof Icon; Kbd: typeof Kbd; Button: typeof Button; SegmentedControl: typeof SegmentedControl; Knob: typeof Knob; Fader: typeof Fader;
      Toggle: typeof Toggle; Select: typeof Select; LedColorPicker: typeof LedColorPicker; LedDisplay: typeof LedDisplay; ConnectionStatus: typeof ConnectionStatus;
      Tag: typeof Tag; ProgressBar: typeof ProgressBar; SearchField: typeof SearchField; CommandPalette: typeof CommandPalette; LibraryTable: typeof LibraryTable;
      PresetGrid: typeof PresetGrid; Panel: typeof Panel; EffectBlock: typeof EffectBlock; SignalChain: typeof SignalChain; PresetEditor: typeof PresetEditor;
      MidiMonitor: typeof MidiMonitor; ICON_NAMES: string[]; LED_COLORS: { value: LedColor; label: string }[];
      fuzzy(query: string, text: string): { score: number; idx: number[] } | null;
    };
  }
}
