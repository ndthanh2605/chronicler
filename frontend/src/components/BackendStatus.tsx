export type BackendConnectionStatus = "idle" | "connecting" | "ok" | "unavailable";

interface BackendStatusProps {
  status: BackendConnectionStatus;
  payload: Record<string, unknown> | null;
  onPing: () => void;
}

/**
 * Reusable surface for backend connectivity state.
 * Phase 4 will reuse this for Vast.ai unreachable errors.
 */
export function BackendStatus({ status, payload, onPing }: BackendStatusProps) {
  return (
    <div>
      <button onClick={onPing} disabled={status === "connecting"}>
        Ping
      </button>
      {status === "connecting" && <p>Connecting to backend…</p>}
      {status === "ok" && payload !== null && (
        <pre>{JSON.stringify(payload, null, 2)}</pre>
      )}
      {status === "unavailable" && (
        <p style={{ color: "red" }}>
          Backend unavailable. Run <code>pnpm dev:backend</code> to build the
          sidecar, then restart the app.
        </p>
      )}
    </div>
  );
}
