export type RecordingState = "idle" | "starting" | "recording" | "stopping";

/** Per-stream VU levels pushed from Rust over the `vu-levels` event (AC3). */
export interface VuLevels {
  mic: number;
  loopback: number;
}

interface RecordControlsProps {
  state: RecordingState;
  meetingId: string | null;
  levels: VuLevels;
  error: string | null;
  onStart: () => void;
  onStop: () => void;
}

/** A single labelled level bar driven by a 0..1 level. */
function VuBar({ label, level }: { label: string; level: number }) {
  const pct = Math.round(Math.min(1, Math.max(0, level)) * 100);
  return (
    <div style={{ marginTop: "0.5rem" }}>
      <span style={{ display: "inline-block", width: "5rem" }}>{label}</span>
      <span
        aria-label={`${label} level`}
        role="meter"
        aria-valuenow={pct}
        aria-valuemin={0}
        aria-valuemax={100}
        style={{
          display: "inline-block",
          width: "12rem",
          height: "0.8rem",
          background: "#eee",
          verticalAlign: "middle",
          borderRadius: "0.2rem",
          overflow: "hidden",
        }}
      >
        <span
          style={{
            display: "block",
            width: `${pct}%`,
            height: "100%",
            background: pct > 85 ? "#d33" : "#3a3",
            transition: "width 60ms linear",
          }}
        />
      </span>
    </div>
  );
}

/**
 * Recording controls + two independent live VU meters (mic, loopback).
 * Levels arrive via a Tauri event channel, never by polling the WAV (AC3).
 */
export function RecordControls({
  state,
  meetingId,
  levels,
  error,
  onStart,
  onStop,
}: RecordControlsProps) {
  const recording = state === "recording" || state === "stopping";
  return (
    <section>
      <button onClick={onStart} disabled={state !== "idle"}>
        {state === "starting" ? "Starting…" : "Record"}
      </button>
      <button onClick={onStop} disabled={!recording}>
        {state === "stopping" ? "Stopping…" : "Stop"}
      </button>

      {recording && meetingId !== null && (
        <p>
          Recording <code>{meetingId}.wav</code>
        </p>
      )}

      <VuBar label="Mic" level={levels.mic} />
      <VuBar label="Loopback" level={levels.loopback} />

      {error !== null && (
        <p style={{ color: "red" }}>Recording error: {error}</p>
      )}
    </section>
  );
}
