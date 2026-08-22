import { useCallback } from "react";
import { useAssistant } from "../contexts/AssistantContext";

export function useAssistantActions() {
  const { sendMessage, clearSession, isLoading, pendingConfirmation, confirmAction, cancelAction } =
    useAssistant();

  const analyzeFile = useCallback(
    (path: string) => sendMessage(`analizar ${path}`),
    [sendMessage],
  );

  const openHistory = useCallback(
    () => sendMessage("abrir historial"),
    [sendMessage],
  );

  const openQuarantine = useCallback(
    () => sendMessage("abrir cuarentena"),
    [sendMessage],
  );

  const getSystemInfo = useCallback(
    () => sendMessage("información del sistema"),
    [sendMessage],
  );

  const getRules = useCallback(
    () => sendMessage("mostrar reglas"),
    [sendMessage],
  );

  const checkVirusTotal = useCallback(
    (hash: string) => sendMessage(`verificar hash ${hash}`),
    [sendMessage],
  );

  return {
    sendMessage,
    clearSession,
    isLoading,
    pendingConfirmation,
    confirmAction,
    cancelAction,
    analyzeFile,
    openHistory,
    openQuarantine,
    getSystemInfo,
    getRules,
    checkVirusTotal,
  };
}
