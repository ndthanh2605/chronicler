import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  BackendStatus,
  type BackendConnectionStatus,
} from "./components/BackendStatus";

function App() {
  const [backendPort, setBackendPort] = useState<number | null>(null);
  const [status, setStatus] = useState<BackendConnectionStatus>("idle");
  const [payload, setPayload] = useState<Record<string, unknown> | null>(null);

  useEffect(() => {
    invoke<number>("get_backend_port")
      .then(setBackendPort)
      .catch(console.error);
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
    } catch {
      setStatus("unavailable");
    }
  }

  return (
    <main>
      <h1>Chronicler</h1>
      <BackendStatus status={status} payload={payload} onPing={handlePing} />
    </main>
  );
}

export default App;
