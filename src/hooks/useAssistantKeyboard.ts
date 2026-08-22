import { useEffect } from "react";
import { useAssistant } from "../contexts/AssistantContext";

export function useAssistantKeyboard() {
  const { togglePanel } = useAssistant();

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      // Ctrl+Shift+A to toggle panel
      if (e.ctrlKey && e.shiftKey && e.key === "A") {
        e.preventDefault();
        togglePanel();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [togglePanel]);
}
