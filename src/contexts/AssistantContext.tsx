import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { tauri } from "../lib/tauri";
import type {
  AssistantMessage,
  AssistantResponse,
  ModelInfo,
} from "../types/assistant";

const STORAGE_KEY = "va-assistant-open";

interface AssistantContextValue {
  messages: AssistantMessage[];
  isOpen: boolean;
  togglePanel: () => void;
  setOpen: (open: boolean) => void;
  sendMessage: (message: string, confirmed?: boolean, pendingId?: string) => Promise<AssistantResponse>;
  clearSession: () => Promise<void>;
  isLoading: boolean;
  error: string | null;
  providerInfo: ModelInfo | null;
  refreshProviderInfo: () => Promise<void>;
  pendingConfirmation: {
    message: string;
    pendingId: string;
  } | null;
  confirmAction: () => Promise<void>;
  cancelAction: () => Promise<void>;
  ysmelActive: boolean;
  fenixActive: boolean;
  silentMode: boolean;
  toggleSilentMode: () => Promise<void>;
}

const AssistantContext = createContext<AssistantContextValue | undefined>(undefined);

export function AssistantProvider({ children }: { children: ReactNode }) {
  const [messages, setMessages] = useState<AssistantMessage[]>([]);
  const [isOpen, setIsOpen] = useState(() => {
    return localStorage.getItem(STORAGE_KEY) === "true";
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [providerInfo, setProviderInfo] = useState<ModelInfo | null>(null);
  const [pendingConfirmation, setPendingConfirmation] = useState<{
    message: string;
    pendingId: string;
  } | null>(null);
  const [ysmelActive, setYsmelActive] = useState(false);
  const [fenixActive, setFenixActive] = useState(false);
  const [silentMode, setSilentMode] = useState(false);

  // Load silent mode on mount
  useEffect(() => {
    void tauri.assistantGetSilentMode().then(setSilentMode).catch(() => undefined);
  }, []);

  // Load history on mount
  useEffect(() => {
    void tauri.assistantGetHistory().then((history) => {
      if (history.length > 0) setMessages(history);
    }).catch(() => undefined);
  }, []);

  // Load provider info
  const refreshProviderInfo = useCallback(async () => {
    try {
      const info = await tauri.assistantGetProviderInfo();
      setProviderInfo(info);
    } catch {
      // Ignore — will show last known info
    }
  }, []);

  useEffect(() => {
    void refreshProviderInfo();
  }, [refreshProviderInfo]);

  // Sync open state to localStorage
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, String(isOpen));
  }, [isOpen]);

  // Sync protocol state from backend
  const syncProtocolState = useCallback(async () => {
    try {
      const ctx = await tauri.assistantGetContext();
      setYsmelActive(ctx.ysmelActive);
      setFenixActive(ctx.fenixActive);
    } catch {
      // Ignore errors during sync
    }
  }, []);

  // Initial sync on mount
  useEffect(() => {
    void syncProtocolState();
  }, [syncProtocolState]);

  const togglePanel = useCallback(() => setIsOpen((prev) => !prev), []);

  const sendMessage = useCallback(
    async (message: string, confirmed?: boolean, pendingId?: string): Promise<AssistantResponse> => {
      setIsLoading(true);
      setError(null);
      try {
        const response = await tauri.assistantSendMessage(message, confirmed, pendingId);

        // Append both user and assistant messages incrementally
        const userMsg: AssistantMessage = {
          id: crypto.randomUUID(),
          role: "user",
          content: message,
          timestamp: new Date().toISOString(),
          requiresConfirmation: false,
        };
        const assistantMsg: AssistantMessage = {
          id: crypto.randomUUID(),
          role: "assistant",
          content: response.message,
          timestamp: new Date().toISOString(),
          intent: response.intent?.type ?? null,
          requiresConfirmation: response.requiresConfirmation,
        };
        setMessages((prev) => [...prev, userMsg, assistantMsg]);

        // Sync protocol state after any message
        await syncProtocolState();

        // Handle confirmation flow
        if (response.requiresConfirmation && response.pendingId) {
          setPendingConfirmation({
            message: response.message,
            pendingId: response.pendingId,
          });
        } else {
          setPendingConfirmation(null);
        }

        return response;
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setError(msg);
        // Add error as assistant message so user sees it
        const errorMsg: AssistantMessage = {
          id: crypto.randomUUID(),
          role: "assistant",
          content: `Error: ${msg}`,
          timestamp: new Date().toISOString(),
          requiresConfirmation: false,
        };
        setMessages((prev) => [...prev, errorMsg]);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [syncProtocolState],
  );

  const clearSession = useCallback(async () => {
    await tauri.assistantClearSession();
    setMessages([]);
    setPendingConfirmation(null);
    setError(null);
  }, []);

  const confirmAction = useCallback(async () => {
    if (!pendingConfirmation) return;
    // Delegate to sendMessage with confirmed=true
    await sendMessage("confirm", true, pendingConfirmation.pendingId);
  }, [pendingConfirmation, sendMessage]);

  const cancelAction = useCallback(async () => {
    await tauri.assistantCancelPending();
    setPendingConfirmation(null);
  }, []);

  const toggleSilentMode = useCallback(async () => {
    try {
      const newState = await tauri.assistantSetSilentMode(!silentMode);
      setSilentMode(newState);
    } catch {
      // Ignore errors
    }
  }, [silentMode]);

  const value = useMemo(
    () => ({
      messages,
      isOpen,
      togglePanel,
      setOpen: setIsOpen,
      sendMessage,
      clearSession,
      isLoading,
      error,
      providerInfo,
      refreshProviderInfo,
      pendingConfirmation,
      confirmAction,
      cancelAction,
      ysmelActive,
      fenixActive,
      silentMode,
      toggleSilentMode,
    }),
    [
      messages,
      isOpen,
      togglePanel,
      sendMessage,
      clearSession,
      isLoading,
      error,
      providerInfo,
      refreshProviderInfo,
      pendingConfirmation,
      confirmAction,
      cancelAction,
      ysmelActive,
      fenixActive,
      silentMode,
      toggleSilentMode,
    ],
  );

  return (
    <AssistantContext.Provider value={value}>
      {children}
    </AssistantContext.Provider>
  );
}

export function useAssistant(): AssistantContextValue {
  const ctx = useContext(AssistantContext);
  if (!ctx) throw new Error("useAssistant must be used within <AssistantProvider>");
  return ctx;
}
