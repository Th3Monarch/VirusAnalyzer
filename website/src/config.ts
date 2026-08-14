import siteConfig from "./site.config.json";

export interface SiteConfig {
  appName: string;
  tagline: string;
  description: string;
  descriptionEs: string;
  fallbackVersion: string;
  siteUrl: string;
  githubOwner: string;
  githubRepository: string;
  discordUrl: string;
}

export const basePath = "/VirusAnalyzer/";
export const basename = basePath.replace(/\/+$/, "");

export const site: SiteConfig = {
  appName: siteConfig.appName,
  tagline: siteConfig.tagline,
  description: siteConfig.description,
  descriptionEs: siteConfig.descriptionEs,
  fallbackVersion: siteConfig.fallbackVersion,
  siteUrl: siteConfig.siteUrl.replace(/\/+$/, ""),
  githubOwner: siteConfig.githubOwner,
  githubRepository: siteConfig.githubRepository,
  discordUrl: siteConfig.discordUrl,
};

export const githubConfigured = Boolean(
  site.githubOwner && site.githubRepository,
);

export const github = {
  configured: githubConfigured,
  owner: site.githubOwner,
  repository: site.githubRepository,
  repoUrl: githubConfigured
    ? `https://github.com/${site.githubOwner}/${site.githubRepository}`
    : null,
  releasesUrl: githubConfigured
    ? `https://github.com/${site.githubOwner}/${site.githubRepository}/releases`
    : null,
  issuesUrl: githubConfigured
    ? `https://github.com/${site.githubOwner}/${site.githubRepository}/issues`
    : null,
  apiBase: githubConfigured
    ? `https://api.github.com/repos/${site.githubOwner}/${site.githubRepository}`
    : null,
};

export const discord = {
  configured: Boolean(site.discordUrl.trim()),
  inviteUrl: site.discordUrl.trim() || null,
};
