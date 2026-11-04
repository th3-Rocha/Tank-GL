import { useCallback, useEffect, useRef, useState } from "react";
import type { MidiMessage, PresetSummary } from "../../design/components/index.d.ts";
import {
  BANKS,
  LETTERS,
  PRESET_IDS,
  backup,
  connect,
  inTauri,
  pickBackupPath,
  programChangeHex,
  readState,
  selectPreset,
  type DeviceInfo,
  type DeviceStatus,
  type Patch,
} from "./api";
import { PatchView } from "./PatchView";

const T = window.Tank;
const POLL_MS = 600;
const LOG_MAX = 200;

// Nomes e cores ainda não foram decifrados do banco; por enquanto só o id.
const PRESETS: PresetSummary[] = PRESET_IDS.map((id) => ({ id, name: `Preset ${id}` }));

type Conn = "connected" | "connecting" | "disconnected" | "error";

function now() {
  return new Date().toLocaleTimeString("pt-BR", { hour12: false }) + "." + String(Date.now() % 1000).padStart(3, "0");
}

function errText(e: unknown) {
  return typeof e === "string" ? e : e instanceof Error ? e.message : JSON.stringify(e);
}

export function App() {
  const [conn, setConn] = useState<Conn>("connecting");
  const [info, setInfo] = useState<DeviceInfo | null>(null);
  const [detail, setDetail] = useState<string>();
  const [status, setStatus] = useState<DeviceStatus | null>(null);
  const [patch, setPatch] = useState<Patch | null>(null);
  const [log, setLog] = useState<MidiMessage[]>([]);
  const [paused, setPaused] = useState(false);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<{ tone: "ok" | "danger"; text: string } | null>(null);

  const pausedRef = useRef(paused);
  pausedRef.current = paused;
  const lastStatus = useRef<string>("");

  const push = useCallback((m: Omit<MidiMessage, "time">) => {
    if (pausedRef.current) return;
    setLog((l) => [...l.slice(-(LOG_MAX - 1)), { time: now(), ...m }]);
  }, []);

  const applyStatus = useCallback(
    (st: DeviceStatus) => {
      setStatus(st);
      const key = `${st.preset}|${st.mode}`;
      if (key !== lastStatus.current) {
        lastStatus.current = key;
        push({ dir: "in", bytes: "READ 0x20000000", text: `Estado: preset ${st.preset ?? "nenhum"} · modo ${st.mode}` });
      }
    },
    [push],
  );

  // Conecta e acompanha o estado (o pedal não avisa quando você pisa num footswitch).
  useEffect(() => {
    let alive = true;
    let timer: number;
    const tick = async () => {
      try {
        if (!info) {
          const i = await connect();
          if (!alive) return;
          setInfo(i);
          push({ dir: "in", bytes: "IDENTITY 0x11", text: `Conectado: ${i.firmware}` });
        }
        const st = await readState();
        if (!alive) return;
        applyStatus(st.status);
        setPatch(st.patch);
        setConn("connected");
        setDetail(undefined);
      } catch (e) {
        if (!alive) return;
        setInfo(null);
        setStatus(null);
        setPatch(null);
        lastStatus.current = "";
        setConn("disconnected");
        setDetail(errText(e));
      }
      timer = window.setTimeout(tick, POLL_MS);
    };
    tick();
    return () => {
      alive = false;
      clearTimeout(timer);
    };
  }, [info, applyStatus, push]);

  const choose = useCallback(
    async (id: string) => {
      if (conn !== "connected") return;
      push({ dir: "out", bytes: programChangeHex(id), text: `Program Change → ${id}`, highlight: true });
      try {
        applyStatus(await selectPreset(id));
      } catch (e) {
        setNotice({ tone: "danger", text: `Não deu para trocar para ${id}: ${errText(e)}` });
      }
    },
    [conn, push, applyStatus],
  );

  const doBackup = async () => {
    const path = await pickBackupPath();
    if (!path) return;
    setBusy(true);
    try {
      const n = await backup(path);
      setNotice({ tone: "ok", text: `Backup salvo: ${n} bytes em ${path}` });
    } catch (e) {
      setNotice({ tone: "danger", text: `Backup falhou: ${errText(e)}` });
    } finally {
      setBusy(false);
    }
  };

  const active = status?.preset ?? null;
  const bank = active ? parseInt(active, 10) : null;
  const letter = active ? active.slice(-1) : null;
  const online = conn === "connected";

  const stepBank = (d: number) => {
    const b = (((bank ?? 1) - 1 + d + BANKS) % BANKS) + 1;
    choose(`${b}${letter ?? "A"}`);
  };

  // Atalhos: 1–4 = A–D, setas = banco.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.ctrlKey || e.metaKey || e.altKey) return;
      const i = "1234".indexOf(e.key);
      if (i >= 0) choose(`${bank ?? 1}${LETTERS[i]}`);
      else if (e.key === "PageUp") stepBank(1);
      else if (e.key === "PageDown") stepBank(-1);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  return (
    <div className="app">
      <header className="topbar">
        <h1 className="brand">Tank Studio</h1>
        <T.ConnectionStatus
          transport="usb"
          state={conn}
          device={info ? info.firmware : "Tank-G"}
          detail={online ? undefined : detail ?? "Procurando a pedaleira…"}
        />
        {!inTauri && <T.Tag tone="warn" icon="alert">Simulado: rodando fora do app</T.Tag>}
        <span className="spacer" />
        <T.Button icon="download" onClick={doBackup} disabled={!online || busy} loading={busy}>
          Fazer backup dos presets
        </T.Button>
      </header>

      {notice && (
        <div className={`notice notice--${notice.tone}`} role="status">
          <T.Icon name={notice.tone === "ok" ? "check" : "alert"} size={16} />
          <span>{notice.text}</span>
          <T.Button variant="ghost" size="sm" icon="close" aria-label="Fechar aviso" onClick={() => setNotice(null)} />
        </div>
      )}

      <main className="layout">
        <T.Panel title="Presets" icon="grid" meta={active ? `No pedal: ${active}` : undefined} className="presets">
          <T.PresetGrid presets={PRESETS} value={active ?? undefined} activeId={active ?? undefined} onSelect={choose} />
          <p className="hint">Clique para tocar o preset na pedaleira.</p>
        </T.Panel>

        <section className="stage">
          <T.Panel title="Na pedaleira" icon="knob" className="stage__panel">
            <div className="hw">
              <div className="hw__display">
                <T.LedDisplay value={bank ? String(bank) : "-"} digits={1} height={96} caption="BANCO" label={`Banco ${bank ?? "nenhum"}`} />
                <div className="hw__bankbtns">
                  <T.Button size="sm" onClick={() => stepBank(1)} disabled={!online} kbd="PgUp">
                    Banco +
                  </T.Button>
                  <T.Button size="sm" onClick={() => stepBank(-1)} disabled={!online} kbd="PgDn">
                    Banco −
                  </T.Button>
                </div>
              </div>
              <div className="hw__info">
                <div className="hw__preset">{active ?? "—"}</div>
                <div className="hw__mode">
                  {status ? (
                    <T.Tag tone={status.mode === "P" ? "amber" : "ok"}>
                      {status.mode === "P" ? "Modo PRESET" : "Modo LIVE"}
                    </T.Tag>
                  ) : (
                    <T.Tag>Sem leitura</T.Tag>
                  )}
                  {status?.mode === "L" && (
                    <span className="muted">No modo LIVE os footswitches ligam e desligam os efeitos.</span>
                  )}
                </div>
              </div>
            </div>

            <div className="footswitches" role="group" aria-label="Footswitches">
              {LETTERS.map((L, i) => {
                const id = `${bank ?? 1}${L}`;
                const on = active === id;
                return (
                  <button
                    key={L}
                    type="button"
                    className={`fs ${on ? "is-on" : ""}`}
                    onClick={() => choose(id)}
                    disabled={!online}
                    aria-pressed={on}
                    title={`Tocar ${id} (tecla ${i + 1})`}
                  >
                    <span className="fs__led" aria-hidden />
                    <span className="fs__letter">{L}</span>
                    <span className="fs__id">{id}</span>
                  </button>
                );
              })}
            </div>
          </T.Panel>

          <T.Panel
            title="Preset atual"
            icon="sliders"
            meta={patch ? <T.Tag size="sm" tone={status?.mode === "L" ? "ok" : "neutral"}>Ao vivo · só leitura</T.Tag> : undefined}
          >
            {patch ? (
              <PatchView patch={patch} live={status?.mode === "L"} />
            ) : (
              <p className="muted">Conecte a pedaleira para ver o preset.</p>
            )}
          </T.Panel>
        </section>

        <T.MidiMonitor
          className="monitor"
          title="Monitor MIDI"
          messages={log}
          paused={paused}
          onTogglePause={() => setPaused((p) => !p)}
          onClear={() => setLog([])}
          height={220}
        />
      </main>
    </div>
  );
}
