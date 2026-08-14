import { useEffect, useState } from "react";
import { getCachedLatestBundle } from "../lib/github";
import type { DownloadBundle } from "../lib/github";
import { github, site } from "../config";

export type ReleaseState =
  | { status: "loading" }
  | { status: "ready"; bundle: DownloadBundle | null };

export function useLatestRelease(): ReleaseState {
  const [state, setState] = useState<ReleaseState>({ status: "loading" });

  useEffect(() => {
    let active = true;
    if (!github.configured) {
      setState({ status: "ready", bundle: null });
      return;
    }
    getCachedLatestBundle(site.githubOwner, site.githubRepository).then(
      (bundle) => {
        if (active) setState({ status: "ready", bundle });
      },
    );
    return () => {
      active = false;
    };
  }, []);

  return state;
}
