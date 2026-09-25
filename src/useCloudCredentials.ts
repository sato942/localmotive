/// Cloud-provider credentials cluster (FE-04): provider list, credential
/// status, key draft, model list, and probe state live here instead of in
/// `App.tsx`. The hook owns every cloud credential IPC command; screens keep
/// receiving the same values through props (P1-35 boundary). Secrets never
/// linger: a failed save clears only the failed attempt (FE-01), and the
/// screens clear drafts on unmount through the exposed setters.
import { useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  errorText,
  responseIsCurrent,
  type CloudModel,
  type CloudProvider,
  type CredentialStatus,
} from "./model";
import {
  persistenceFailureNote,
  persistRecord,
  readRecord,
  readSetting,
} from "./persistence";

export interface CloudCredentials {
  providers: CloudProvider[];
  providerId: string;
  provider: CloudProvider | null;
  credential: CredentialStatus | null;
  keyDraft: string;
  setKeyDraft: React.Dispatch<React.SetStateAction<string>>;
  cloudModels: CloudModel[];
  cloudModel: string;
  cloudCheck: string;
  loadCloud: (nextProvider?: string) => Promise<void>;
  switchProvider: (next: string) => Promise<void>;
  onProviderTabKey: (event: React.KeyboardEvent<HTMLButtonElement>, index: number) => void;
  saveKey: () => Promise<void>;
  forgetKey: () => Promise<void>;
  openRouterLogin: () => Promise<void>;
  probeCloud: () => Promise<void>;
  chooseCloudModel: (id: string) => void;
}

export function useCloudCredentials(deps: {
  notify: (message: string) => void;
  setBusyState: (name: string) => void;
  isBrowserPreview: boolean;
}): CloudCredentials {
  const { notify, setBusyState, isBrowserPreview } = deps;
  const [providers, setProviders] = useState<CloudProvider[]>([]);
  const [providerId, setProviderId] = useState(() => readSetting("cloud-provider") || "openrouter");
  const [credential, setCredential] = useState<CredentialStatus | null>(null);
  const [keyDraft, setKeyDraft] = useState("");
  const [cloudModels, setCloudModels] = useState<CloudModel[]>([]);
  const [cloudModel, setCloudModel] = useState(() => readSetting("cloud-model"));
  const [cloudCheck, setCloudCheck] = useState("");
  const cloudSeq = useRef(0);
  const providerIdRef = useRef(providerId);
  providerIdRef.current = providerId;

  const provider = providers.find((entry) => entry.id === providerId) ?? null;

  async function loadCloud(nextProvider = providerId) {
    const sequence = ++cloudSeq.current;
    try {
      const list = providers.length ? providers : await invoke<CloudProvider[]>("cloud_providers");
      if (sequence !== cloudSeq.current) return;
      if (!providers.length) setProviders(list);
      const status = await invoke<CredentialStatus>("cloud_credential_status", { provider: nextProvider });
      if (!responseIsCurrent(sequence, cloudSeq.current, nextProvider, providerIdRef.current)) return;
      setCredential(status);
      setCloudModels([]);
      setCloudCheck("");
      setCloudModel("");
      if (status.configured) {
        try {
          const modelsList = await invoke<CloudModel[]>("cloud_list_models", { provider: nextProvider });
          if (!responseIsCurrent(sequence, cloudSeq.current, nextProvider, providerIdRef.current)) return;
          setCloudModels(modelsList);
          const fallback = list.find((entry) => entry.id === nextProvider)?.defaultModel ?? "";
          const stored = readRecord(`cloud-model:${nextProvider}`);
          const chosen = stored && modelsList.some((m) => m.id === stored) ? stored : modelsList.some((m) => m.id === fallback) ? fallback : (modelsList[0]?.id ?? fallback);
          setCloudModel(chosen);
        } catch (error) {
          if (responseIsCurrent(sequence, cloudSeq.current, nextProvider, providerIdRef.current)) setCloudCheck(errorText(error));
        }
      }
    } catch (error) {
      if (sequence !== cloudSeq.current) return;
      if (isBrowserPreview) {
        setProviders([
          { id: "openrouter", label: "OpenRouter", baseUrl: "https://openrouter.ai/api/v1", keyPrefixHint: "sk-or-", consoleUrl: "https://openrouter.ai/settings/keys", supportsOauth: true, defaultModel: "anthropic/claude-sonnet-4.6", listsModels: true },
          { id: "anthropic", label: "Anthropic", baseUrl: "https://api.anthropic.com/v1", keyPrefixHint: "sk-ant-", consoleUrl: "https://platform.claude.com/settings/keys", supportsOauth: false, defaultModel: "claude-sonnet-4-6", listsModels: true },
          { id: "openai", label: "OpenAI", baseUrl: "https://api.openai.com/v1", keyPrefixHint: "sk-", consoleUrl: "https://platform.openai.com/api-keys", supportsOauth: false, defaultModel: "gpt-5", listsModels: true },
          { id: "gemini", label: "Google Gemini", baseUrl: "https://generativelanguage.googleapis.com/v1beta/openai", keyPrefixHint: "AIza", consoleUrl: "https://aistudio.google.com/apikey", supportsOauth: false, defaultModel: "gemini-2.5-pro", listsModels: true },
        ]);
        setCredential({ provider: nextProvider, configured: false, masked: "" });
      } else {
        notify(errorText(error));
      }
    }
  }

  async function switchProvider(next: string) {
    setProviderId(next);
    if (!persistRecord("cloud-provider", next)) {
      notify(persistenceFailureNote("The provider choice"));
    }
    setKeyDraft("");
    // FE-03: clear provider-specific presentation at switch start; a slow
    // previous provider can never relabel the new tab while it loads.
    cloudSeq.current += 1;
    setCredential(null);
    setCloudModels([]);
    setCloudCheck("");
    await loadCloud(next);
  }

  /// WAI-ARIA tabs keyboard pattern (audit FE-13): arrows cycle, Home/End
  /// jump, and focus follows the selection.
  function onProviderTabKey(
    event: React.KeyboardEvent<HTMLButtonElement>,
    index: number,
  ) {
    let next = index;
    if (event.key === "ArrowRight") next = (index + 1) % providers.length;
    else if (event.key === "ArrowLeft") next = (index - 1 + providers.length) % providers.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = providers.length - 1;
    else return;
    event.preventDefault();
    const target = providers[next];
    if (!target) return;
    switchProvider(target.id);
    document.getElementById(`provider-tab-${target.id}`)?.focus();
  }

  async function saveKey() {
    if (!keyDraft.trim()) return;
    const forProvider = providerId;
    const attempted = keyDraft;
    const sequence = ++cloudSeq.current;
    setBusyState("cloud");
    try {
      const status = await invoke<CredentialStatus>("cloud_save_credential", { provider: forProvider, secret: keyDraft });
      if (!responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) return;
      setKeyDraft("");
      setCredential(status);
      notify(`${providers.find((entry) => entry.id === forProvider)?.label ?? forProvider} key stored in Windows Credential Manager.`);
      await loadCloud(forProvider);
    } catch (error) {
      if (!responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) return;
      // FE-01: a failed save must not leave the typed secret in frontend state.
      setKeyDraft((current) => (current === attempted ? "" : current));
      notify(errorText(error));
    } finally {
      setBusyState("");
    }
  }

  async function forgetKey() {
    const forProvider = providerId;
    const sequence = ++cloudSeq.current;
    setBusyState("cloud");
    try {
      const status = await invoke<CredentialStatus>("cloud_clear_credential", { provider: forProvider });
      if (!responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) return;
      setCredential(status);
      setCloudModels([]);
      setCloudCheck("");
      notify(`${providers.find((entry) => entry.id === forProvider)?.label ?? forProvider} key removed from Windows Credential Manager.`);
    } catch (error) {
      if (responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) notify(errorText(error));
    } finally {
      setBusyState("");
    }
  }

  async function openRouterLogin() {
    const sequence = ++cloudSeq.current;
    setBusyState("oauth");
    notify("Finish signing in to OpenRouter in your browser. Localmotive is waiting on a local callback.");
    try {
      const status = await invoke<CredentialStatus>("cloud_openrouter_login");
      if (!responseIsCurrent(sequence, cloudSeq.current, "openrouter", providerIdRef.current)) return;
      setCredential(status);
      notify("OpenRouter connected. A user-controlled key was issued and stored in Windows Credential Manager.");
      await loadCloud("openrouter");
    } catch (error) {
      // A cancelled/timed-out sign-in must not relabel another provider's tab.
      if (responseIsCurrent(sequence, cloudSeq.current, "openrouter", providerIdRef.current)) notify(errorText(error));
    } finally {
      setBusyState("");
    }
  }

  async function probeCloud() {
    const forProvider = providerId;
    const forModel = cloudModel;
    const sequence = ++cloudSeq.current;
    setBusyState("probe");
    setCloudCheck("Contacting provider…");
    try {
      const reply = await invoke<string>("cloud_probe", { provider: forProvider, model: forModel });
      if (!responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) return;
      setCloudCheck(`Connected · ${forModel} replied “${reply.trim().slice(0, 40)}”`);
    } catch (error) {
      if (responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) setCloudCheck(errorText(error));
    } finally {
      setBusyState("");
    }
  }

  function chooseCloudModel(id: string) {
    setCloudModel(id);
    const savedGlobally = persistRecord("cloud-model", id);
    const savedForProvider = persistRecord(`cloud-model:${providerId}`, id);
    if (!savedGlobally || !savedForProvider) {
      notify(persistenceFailureNote("The advisor model choice"));
    }
  }

  return {
    providers,
    providerId,
    provider,
    credential,
    keyDraft,
    setKeyDraft,
    cloudModels,
    cloudModel,
    cloudCheck,
    loadCloud,
    switchProvider,
    onProviderTabKey,
    saveKey,
    forgetKey,
    openRouterLogin,
    probeCloud,
    chooseCloudModel,
  };
}
