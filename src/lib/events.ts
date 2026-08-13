import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { ScanEvent, ScanHistoryEntry, ScanProgress } from "../types";

/**
 * Escucha los eventos de escaneo del backend y los normaliza en un único
 * flujo `ScanEvent`. Devuelve la función para dejar de escuchar.
 */
export async function subscribeScanEvents(onEvent: (event: ScanEvent) => void): Promise<UnlistenFn> {
  const unlisteners: UnlistenFn[] = [];

  unlisteners.push(
    await listen<ScanProgress>("scan-progress", (e) => {
      onEvent({ type: "progress", progress: e.payload });
    }),
  );
  unlisteners.push(
    await listen<{ scanId: string; entry: ScanHistoryEntry }>("scan-completed", (e) => {
      onEvent({ type: "completed", scanId: e.payload.scanId, entry: e.payload.entry });
    }),
  );
  unlisteners.push(
    await listen<{ scanId: string; message: string }>("scan-error", (e) => {
      onEvent({ type: "error", scanId: e.payload.scanId, message: e.payload.message });
    }),
  );
  unlisteners.push(
    await listen<{ scanId: string }>("scan-cancelled", (e) => {
      onEvent({ type: "cancelled", scanId: e.payload.scanId });
    }),
  );

  return () => unlisteners.forEach((fn) => fn());
}

export interface DragDropState {
  over: boolean;
  paths: string[];
}

/**
 * Drag & drop de archivos/carpetas del sistema hacia la ventana.
 */
export function onDragDrop(
  onDrop: (paths: string[]) => void,
  onState?: (state: DragDropState) => void,
): () => void {
  let unlisten: UnlistenFn | null = null;
  let disposed = false;

  void getCurrentWebview()
    .onDragDropEvent((event) => {
      const type = event.payload.type;
      if (type === "over") {
        onState?.({ over: true, paths: [] });
      } else if (type === "drop") {
        onState?.({ over: false, paths: event.payload.paths });
        onDrop(event.payload.paths);
      } else if (type === "leave") {
        onState?.({ over: false, paths: [] });
      }
    })
    .then((fn) => {
      if (disposed) {
        fn();
      } else {
        unlisten = fn;
      }
    })
    .catch(() => undefined);

  return () => {
    disposed = true;
    unlisten?.();
  };
}
