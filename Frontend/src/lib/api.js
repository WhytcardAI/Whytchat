// Polyfill for development in browser without Tauri
const mockInvoke = async (cmd, args) => {
  console.log(`[Mock] invoke: ${cmd}`, args);
  if (cmd === "health") return "ok";
  if (cmd === "chat") return { reply: "This is a mock response from the browser." };
  return null;
};

const mockListen = (event, _callback) => {
  console.log(`[Mock] listen: ${event}`);
  return () => {};
};

const tauri = window.__TAURI__ || {
  core: { invoke: mockInvoke },
  event: { listen: mockListen },
};

const invoke = tauri.core.invoke;
const listen = tauri.event.listen;

export const api = {
  health: () => invoke("health"),
  installServer: (backend = "cpu") => invoke("install_server", { backend }),
  downloadModel: (expectedSha256 = null) => invoke("download_model", { expectedSha256 }),
  startServer: () => invoke("start_server"),
  stopServer: () => invoke("stop_server"),
  serverReady: () => invoke("server_ready"),
  chat: (messages) => invoke("chat", { input: { messages } }),
  chatMulti: (agents, history, synthSystemPrompt) =>
    invoke("chat_multi", {
      input: { agents, history, synth_system_prompt: synthSystemPrompt },
    }),

  chatFusion: (historyA, historyB, commonPrompt) =>
    invoke("chat_fusion", {
      input: { history_a: historyA, history_b: historyB, common_prompt: commonPrompt },
    }),

  generateResponseWithContext: (contextHistory, userPrompt, agentName) =>
    invoke("generate_response_with_context", {
      input: {
        context_history: contextHistory,
        user_prompt: userPrompt,
        agent_name: agentName,
      },
    }),

  ingestFile: (path) => invoke("ingest_file", { path }),
  queryRag: (query, limit = 3) => invoke("query_rag", { query, limit }),
  searchWeb: (query) => invoke("search_web", { query }),

  on: (event, callback) => listen(event, callback),
};
