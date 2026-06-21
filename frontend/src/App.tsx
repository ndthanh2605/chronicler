import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  BackendStatus,
  type BackendConnectionStatus,
} from "./components/BackendStatus";
import {
  RecordControls,
  type RecordingState,
  type VuLevels,
} from "./components/RecordControls";

function App() {
  const [backendPort, setBackendPort] = useState<number | null>(null);
  const [status, setStatus] = useState<BackendConnectionStatus>("idle");
  const [payload, setPayload] = useState<Record<string, unknown> | null>(null);

  const [recState, setRecState] = useState<RecordingState>("idle");
  const [meetingId, setMeetingId] = useState<string | null>(null);
  const [levels, setLevels] = useState<VuLevels>({ mic: 0, loopback: 0 });
  const [recError, setRecError] = useState<string | null>(null);

  useEffect(() => {
    invoke<number>("get_backend_port")
      .then(setBackendPort)
      .catch(console.error);
  }, []);

  // Subscribe to the live VU level stream from Rust (AC3). The listener is
  // always active; levels stay at zero until recording emits them.
  useEffect(() => {
    const unlisten = listen<VuLevels>("vu-levels", (event) => {
      setLevels(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn()).catch(console.error);
    };
  }, []);

  async function handlePing() {
    if (backendPort === null) return;
    setStatus("connecting");
    try {
      const res = await fetch(`http://127.0.0.1:${backendPort}/health`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = (await res.json()) as Record<string, unknown>;
      setPayload(data);
      setStatus("ok");
    } catch (err) {
      console.error("Ping failed:", err);
      setStatus("unavailable");
    }
  }

  async function handleStartRecording() {
    setRecError(null);
    setRecState("starting");
    try {
      const id = await invoke<string>("start_recording");
      setMeetingId(id);
      setRecState("recording");
    } catch (err) {
      console.error("start_recording failed:", err);
      setRecError(String(err));
      setRecState("idle");
    }
  }

  async function handleStopRecording() {
    setRecState("stopping");
    try {
      await invoke("stop_recording");
    } catch (err) {
      console.error("stop_recording failed:", err);
      setRecError(String(err));
    } finally {
      setLevels({ mic: 0, loopback: 0 });
      setRecState("idle");
    }
  }

  return (
    <main>
      <h1>Chronicler</h1>
      <RecordControls
        state={recState}
        meetingId={meetingId}
        levels={levels}
        error={recError}
        onStart={handleStartRecording}
        onStop={handleStopRecording}
      />
      <BackendStatus status={status} payload={payload} onPing={handlePing} />
    </main>
  );
}

export default App;
