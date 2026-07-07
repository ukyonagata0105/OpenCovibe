// Provider presets for the Mofu CLI app-server compatibility route.
// Mofu App connects only to `mofu`; the selected provider is passed through that CLI route.

export interface CodexProviderPreset {
  id: string;
  name: string;
  description: string;
  /** OpenAI-compatible base URL implementing the Responses API. */
  base_url: string;
  /** Env var name the API key is injected under (we set it at spawn). */
  env_key: string;
  /** Suggested model id (provider-side, OpenAI format). Empty → user fills / Codex default. */
  model: string;
  key_placeholder: string;
  /** Keyless (local) providers like Ollama need no API key. */
  keyless?: boolean;
  /** Custom = user supplies base_url/env_key/model. */
  custom?: boolean;
  docs_url?: string;
}

export const CODEX_PROVIDER_PRESETS: CodexProviderPreset[] = [
  {
    id: "lm-studio",
    name: "LM Studio",
    description: "Local OpenAI-compatible endpoint served by LM Studio",
    base_url: "http://172.17.30.209:1234/v1",
    env_key: "LM_STUDIO_API_KEY",
    model: "qwen3.5-0.8b-mlx",
    key_placeholder: "lm-studio",
    custom: true,
  },
];
