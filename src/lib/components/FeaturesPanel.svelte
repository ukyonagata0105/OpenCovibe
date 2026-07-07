<script lang="ts">
  import * as api from "$lib/api";
  import type { CodexFeature } from "$lib/api";
  import { dbg, dbgWarn } from "$lib/utils/debug";
  import { currentLocale, t } from "$lib/i18n/index.svelte";

  let {
    runId,
    sessionAlive = false,
    onClose,
  }: {
    runId: string;
    sessionAlive?: boolean;
    onClose: () => void;
  } = $props();

  let loading = $state(false);
  let features = $state<CodexFeature[]>([]);
  let togglingName = $state<string | null>(null);
  let error = $state("");
  let loaded = $state(false);

  const FEATURE_LABELS_JA: Record<string, string> = {
    memories: "メモリ",
    network_proxy: "ネットワークプロキシ",
    prevent_sleep_while_running: "実行中のスリープ防止",
    shell_tool: "シェルツール",
    secret_auth_storage: "認証情報の安全な保存",
    unified_exec: "統合コマンド実行",
    shell_snapshot: "シェルスナップショット",
    hooks: "Hooks",
    enable_request_compression: "リクエスト圧縮",
    multi_agent: "マルチエージェント",
    apps: "アプリ連携",
    tool_suggest: "ツール候補",
    plugins: "プラグイン",
    in_app_browser: "アプリ内ブラウザ",
    browser_use: "ブラウザ操作",
    browser_use_full_cdp_access: "ブラウザ操作 CDP フルアクセス",
    browser_use_external: "外部ブラウザ操作",
    computer_use: "コンピュータ操作",
    plugin_sharing: "プラグイン共有",
    image_generation: "画像生成",
    skill_mcp_dependency_install: "Skill/MCP 依存関係インストール",
    mentions_v2: "メンション v2",
    guardian_approval: "Guardian 承認",
    goals: "目標",
    tool_call_mcp_elicitation: "MCP ツール確認",
    personality: "人格プリセット",
    fast_mode: "高速モード",
    auto_compaction: "自動圧縮",
    remote_compaction_v2: "リモート圧縮 v2",
    workspace_dependencies: "ワークスペース依存関係",
    shell_zsh_fork: "zsh シェル分岐",
    unified_exec_zsh_fork: "統合実行 zsh 分岐",
    deferred_executor: "遅延実行",
    code_mode: "コードモード",
    code_mode_only: "コードモード専用",
    standalone_web_search: "単体 Web 検索",
    runtime_metrics: "ランタイム指標",
    local_thread_store_compression: "ローカルスレッド保存圧縮",
    chronicle: "Chronicle",
    apply_patch_streaming_events: "apply_patch ストリーミングイベント",
    exec_permission_approvals: "実行権限承認",
    request_permissions_tool: "権限リクエストツール",
    respect_system_proxy: "システムプロキシ尊重",
    multi_agent_v2: "マルチエージェント v2",
    enable_fanout: "ファンアウト",
    enable_mcp_apps: "MCP アプリ",
    non_prefixed_mcp_tool_names: "MCP ツール名の接頭辞省略",
    remote_plugin: "リモートプラグイン",
    imagegenext: "画像生成拡張",
    item_ids: "アイテム ID",
    default_mode_request_user_input: "既定モードの入力リクエスト",
    terminal_visualization_instructions: "ターミナル可視化指示",
    token_budget: "トークン予算",
    rollout_budget: "ロールアウト予算",
    current_time_reminder: "現在時刻リマインダー",
    sleep_tool: "スリープツール",
    auth_elicitation: "認証確認",
    artifact: "成果物",
    realtime_conversation: "リアルタイム会話",
    use_agent_identity: "エージェント ID",
    web_search_request: "Web 検索リクエスト",
    web_search_cached: "キャッシュ済み Web 検索",
    use_legacy_landlock: "旧 Landlock",
  };

  const FEATURE_DESCRIPTIONS_JA: Record<string, string> = {
    memories: "会話から新しいメモリを作成し、新しい会話に関連メモリを取り込めるようにします。",
    network_proxy: "ネットワークアクセスがあるサンドボックスセッションにプロキシ制限を適用します。",
    prevent_sleep_while_running: "Mofu CLI がスレッドを実行している間、Mac のスリープを防ぎます。",
  };

  function featureLabel(feature: CodexFeature): string {
    if (currentLocale() === "ja") {
      return FEATURE_LABELS_JA[feature.name] ?? feature.displayName ?? feature.name;
    }
    return feature.displayName || feature.name;
  }

  function featureDescription(feature: CodexFeature): string {
    if (currentLocale() === "ja") {
      return FEATURE_DESCRIPTIONS_JA[feature.name] ?? feature.description ?? "";
    }
    return feature.description ?? "";
  }

  // Stage policy: which lifecycle stages a user is allowed to toggle via config.
  // - beta / stable: real user-facing flags → render with a toggle (primary).
  // - underDevelopment / deprecated: shown read-only (visible for transparency,
  //   but writing `[features].<name>` for a half-built or sunsetting flag would
  //   set an option the CLI isn't ready to honor) — no toggle.
  // - removed: the flag no longer does anything → hidden entirely.
  function isToggleable(stage: string): boolean {
    return stage === "beta" || stage === "stable";
  }

  // Toggleable (beta first, then stable) followed by read-only dev/deprecated.
  // `removed` is dropped. Stable sort within a group keeps Codex's order.
  const STAGE_ORDER: Record<string, number> = {
    beta: 0,
    stable: 1,
    underDevelopment: 2,
    deprecated: 3,
  };
  const visibleFeatures = $derived(
    features
      .filter((f) => f.stage !== "removed")
      .slice()
      .sort((a, b) => (STAGE_ORDER[a.stage] ?? 9) - (STAGE_ORDER[b.stage] ?? 9)),
  );

  function stageBadgeClass(stage: string): string {
    switch (stage) {
      case "beta":
        return "bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/30";
      case "stable":
        return "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30";
      case "underDevelopment":
        return "bg-sky-500/10 text-sky-600 dark:text-sky-400 border-sky-500/30";
      case "deprecated":
        return "bg-muted text-muted-foreground border-border";
      default:
        return "bg-muted text-muted-foreground border-border";
    }
  }

  function stageLabel(stage: string): string {
    switch (stage) {
      case "beta":
        return t("features_stageBeta");
      case "stable":
        return t("features_stageStable");
      case "underDevelopment":
        return t("features_stageDev");
      case "deprecated":
        return t("features_stageDeprecated");
      default:
        return stage;
    }
  }

  async function refresh() {
    if (!sessionAlive) return;
    loading = true;
    error = "";
    try {
      dbg("features", "list", { runId });
      const res = await api.listCodexFeatures(runId);
      features = res.data ?? [];
      loaded = true;
      dbg("features", "listed", { count: features.length });
    } catch (e) {
      dbgWarn("features", "list failed", e);
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function toggle(feature: CodexFeature) {
    if (!isToggleable(feature.stage)) return;
    const newEnabled = !feature.enabled;
    togglingName = feature.name;
    error = "";
    try {
      dbg("features", "toggle", { name: feature.name, enabled: newEnabled });
      await api.setCodexFeature(feature.name, newEnabled);
      // Optimistic — durable write takes effect next session, not live.
      features = features.map((f) => (f.name === feature.name ? { ...f, enabled: newEnabled } : f));
    } catch (e) {
      dbgWarn("features", "toggle failed", e);
      error = String(e);
    } finally {
      togglingName = null;
    }
  }

  // Load once when the panel mounts with a live session.
  $effect(() => {
    if (sessionAlive && !loaded && !loading) {
      void refresh();
    }
  });
</script>

<div class="rounded-lg border border-border bg-background shadow-lg w-96 animate-fade-in">
  <!-- Header -->
  <div class="flex items-center justify-between px-3 py-2 border-b border-border">
    <span class="text-xs font-semibold text-foreground">{t("features_title")}</span>
    <div class="flex items-center gap-1">
      <button
        class="rounded p-1 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors disabled:opacity-50"
        disabled={loading || !sessionAlive}
        onclick={refresh}
        title={t("features_refresh")}
      >
        <svg
          class="h-3.5 w-3.5 {loading ? 'animate-spin' : ''}"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
          <path d="M3 3v5h5" />
          <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
          <path d="M16 16h5v5" />
        </svg>
      </button>
      <button
        class="rounded p-1 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
        onclick={onClose}
        title={t("common_close")}
      >
        <svg
          class="h-3.5 w-3.5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M18 6 6 18" /><path d="m6 6 12 12" />
        </svg>
      </button>
    </div>
  </div>

  <!-- "Next session" hint -->
  <div class="px-3 py-1.5 border-b border-border/50 text-[10px] text-muted-foreground">
    {t("features_nextSessionHint")}
  </div>

  <!-- Feature list -->
  <div class="max-h-80 overflow-y-auto">
    {#if !sessionAlive}
      <div class="px-3 py-4 text-center text-xs text-muted-foreground">
        {t("features_noSession")}
      </div>
    {:else if loading && !loaded}
      <div class="px-3 py-4 text-center text-xs text-muted-foreground">
        {t("features_loading")}
      </div>
    {:else if visibleFeatures.length === 0}
      <div class="px-3 py-4 text-center text-xs text-muted-foreground">
        {t("features_none")}
      </div>
    {:else}
      {#each visibleFeatures as feature (feature.name)}
        {@const toggleable = isToggleable(feature.stage)}
        {@const overridden = feature.enabled !== feature.defaultEnabled}
        {@const description = featureDescription(feature)}
        <div class="flex items-start gap-2 px-3 py-2 border-b border-border/50 last:border-b-0">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-1.5 flex-wrap">
              <span class="text-xs font-medium text-foreground">
                {featureLabel(feature)}
              </span>
              <span
                class="rounded-full border px-1.5 py-px text-[9px] font-medium uppercase tracking-wide {stageBadgeClass(
                  feature.stage,
                )}"
              >
                {stageLabel(feature.stage)}
              </span>
              {#if overridden}
                <span
                  class="rounded-full border border-border bg-muted px-1.5 py-px text-[9px] font-medium text-muted-foreground"
                  title={t("features_overriddenTitle")}
                >
                  {t("features_overridden")}
                </span>
              {/if}
            </div>
            {#if description}
              <div class="mt-0.5 text-[10px] leading-snug text-muted-foreground">
                {description}
              </div>
            {/if}
          </div>

          <!-- Toggle (beta/stable) or read-only state pill (dev/deprecated) -->
          <div class="shrink-0 pt-0.5">
            {#if toggleable}
              <button
                type="button"
                role="switch"
                aria-checked={feature.enabled}
                disabled={togglingName === feature.name}
                onclick={() => toggle(feature)}
                title={feature.enabled ? t("features_disable") : t("features_enable")}
                class="relative inline-flex h-4 w-7 items-center rounded-full transition-colors disabled:opacity-50 {feature.enabled
                  ? 'bg-emerald-500'
                  : 'bg-muted-foreground/30'}"
              >
                <span
                  class="inline-block h-3 w-3 transform rounded-full bg-white shadow transition-transform {feature.enabled
                    ? 'translate-x-3.5'
                    : 'translate-x-0.5'}"
                ></span>
              </button>
            {:else}
              <span
                class="text-[10px] font-medium {feature.enabled
                  ? 'text-emerald-600 dark:text-emerald-400'
                  : 'text-muted-foreground'}"
                title={t("features_readonlyTitle")}
              >
                {feature.enabled ? t("features_on") : t("features_off")}
              </span>
            {/if}
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Error -->
  {#if error}
    <div class="px-3 py-2 border-t border-destructive/20 bg-destructive/5 text-xs text-destructive">
      {error}
    </div>
  {/if}
</div>
