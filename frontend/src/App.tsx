import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [response, setResponse] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function handlePing() {
    setError(null);
    try {
      const result = await invoke<string>("ping");
      setResponse(result);
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <main>
      <h1>Chronicler</h1>
      <button onClick={handlePing}>Ping</button>
      {response && <p>{response}</p>}
      {error && <p style={{ color: "red" }}>{error}</p>}
    </main>
  );
}

export default App;
