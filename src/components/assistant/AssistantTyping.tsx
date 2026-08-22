import { Bot } from "lucide-react";

export function AssistantTyping() {
  return (
    <div className="mb-3 flex gap-2">
      <div className="flex size-7 shrink-0 items-center justify-center rounded-full bg-surface-2 text-muted">
        <Bot className="size-3.5" />
      </div>
      <div className="flex items-center gap-1 rounded-2xl rounded-bl-md bg-surface-2 px-3.5 py-3">
        <span className="assistant-typing-dot size-1.5 rounded-full bg-muted/50" />
        <span className="assistant-typing-dot size-1.5 rounded-full bg-muted/50" style={{ animationDelay: "0.15s" }} />
        <span className="assistant-typing-dot size-1.5 rounded-full bg-muted/50" style={{ animationDelay: "0.3s" }} />
      </div>
    </div>
  );
}
