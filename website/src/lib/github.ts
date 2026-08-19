export interface ReleaseAsset {
  name: string;
  browser_download_url: string;
  size: number;
}

export interface Release {
  tag_name: string;
  name: string | null;
  published_at: string;
  html_url: string;
  body: string | null;
  assets: ReleaseAsset[];
  prerelease: boolean;
  draft: boolean;
}

export type DownloadKind = "setup" | "portable" | "executable" | "deb" | "appimage" | "dmg";

export interface DownloadAsset {
  name: string;
  url: string;
  size: number;
  kind: DownloadKind;
}

export interface Checksum {
  targetName: string;
  hash: string;
}

export interface DownloadBundle {
  version: string;
  tagName: string;
  releaseUrl: string;
  publishedAt: string;
  body: string | null;
  downloads: DownloadAsset[];
  checksums: Checksum[];
}

const SETUP_RE = /-Setup\.exe$/i;
const PORTABLE_RE = /-Portable\.zip$/i;
const EXECUTABLE_RE = /\.exe$/i;
const DEB_RE = /\.deb$/i;
const APPIMAGE_RE = /\.AppImage$/i;
const DMG_RE = /\.dmg$/i;
const CHECKSUM_RE = /\.sha256$/i;

function isSetup(name: string): boolean {
  return SETUP_RE.test(name);
}

function isPortable(name: string): boolean {
  return PORTABLE_RE.test(name);
}

function isExecutable(name: string): boolean {
  return EXECUTABLE_RE.test(name) && !isSetup(name) && !isPortable(name);
}

export function classifyAsset(asset: ReleaseAsset): DownloadAsset | null {
  if (isSetup(asset.name)) {
    return { name: asset.name, url: asset.browser_download_url, size: asset.size, kind: "setup" };
  }
  if (isPortable(asset.name)) {
    return { name: asset.name, url: asset.browser_download_url, size: asset.size, kind: "portable" };
  }
  if (isExecutable(asset.name)) {
    return { name: asset.name, url: asset.browser_download_url, size: asset.size, kind: "executable" };
  }
  if (DEB_RE.test(asset.name)) {
    return { name: asset.name, url: asset.browser_download_url, size: asset.size, kind: "deb" };
  }
  if (APPIMAGE_RE.test(asset.name)) {
    return { name: asset.name, url: asset.browser_download_url, size: asset.size, kind: "appimage" };
  }
  if (DMG_RE.test(asset.name)) {
    return { name: asset.name, url: asset.browser_download_url, size: asset.size, kind: "dmg" };
  }
  return null;
}

async function getJson<T>(url: string): Promise<T | null> {
  try {
    const res = await fetch(url, {
      headers: {
        Accept: "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
      },
    });
    if (!res.ok) return null;
    return (await res.json()) as T;
  } catch {
    return null;
  }
}

async function fetchChecksumText(
  url: string,
): Promise<string | null> {
  try {
    const res = await fetch(url);
    if (!res.ok) return null;
    return await res.text();
  } catch {
    return null;
  }
}

export async function fetchLatestRelease(
  owner: string,
  repo: string,
): Promise<Release | null> {
  return getJson<Release>(
    `https://api.github.com/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/latest`,
  );
}

export async function fetchRecentReleases(
  owner: string,
  repo: string,
  perPage = 10,
): Promise<Release[] | null> {
  return getJson<Release[]>(
    `https://api.github.com/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases?per_page=${perPage}`,
  );
}

export function versionFromTag(tag: string): string {
  return tag.replace(/^v/i, "");
}

export async function buildDownloadBundle(
  release: Release,
): Promise<DownloadBundle> {
  const downloads = release.assets
    .map(classifyAsset)
    .filter((a): a is DownloadAsset => a !== null);

  const checksums: Checksum[] = [];
  for (const asset of release.assets) {
    if (!CHECKSUM_RE.test(asset.name)) continue;
    const text = await fetchChecksumText(asset.browser_download_url);
    if (!text) continue;
    const firstLine = text.split(/\r?\n/)[0]?.trim();
    const hash = firstLine?.match(/\b[0-9a-fA-F]{64}\b/)?.[0];
    if (!hash) continue;
    checksums.push({
      targetName: asset.name.replace(CHECKSUM_RE, ""),
      hash: hash.toLowerCase(),
    });
  }

  return {
    version: versionFromTag(release.tag_name),
    tagName: release.tag_name,
    releaseUrl: release.html_url,
    publishedAt: release.published_at,
    body: release.body,
    downloads,
    checksums,
  };
}

export async function getLatestDownloadBundle(
  owner: string,
  repo: string,
): Promise<DownloadBundle | null> {
  const latest = await fetchLatestRelease(owner, repo);
  if (latest && latest.assets.some((a) => !CHECKSUM_RE.test(a.name))) {
    return buildDownloadBundle(latest);
  }

  const recent = await fetchRecentReleases(owner, repo, 10);
  if (recent) {
    for (const release of recent) {
      if (release.prerelease || release.draft) continue;
      if (release.assets.some((a) => !CHECKSUM_RE.test(a.name))) {
        return buildDownloadBundle(release);
      }
    }
  }

  if (latest) return buildDownloadBundle(latest);
  return null;
}

let cachedBundle: Promise<DownloadBundle | null> | null = null;

export function getCachedLatestBundle(
  owner: string,
  repo: string,
): Promise<DownloadBundle | null> {
  if (!cachedBundle) {
    cachedBundle = getLatestDownloadBundle(owner, repo).catch(() => null);
  }
  return cachedBundle;
}

export function resetCachedBundle(): void {
  cachedBundle = null;
}
