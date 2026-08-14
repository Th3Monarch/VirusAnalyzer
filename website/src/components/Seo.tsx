import { useEffect } from "react";
import { site } from "../config";

interface SeoProps {
  title: string;
  description?: string;
  path: string;
  type?: string;
}

function upsertMeta(
  attr: "name" | "property",
  key: string,
  content: string,
): void {
  let el = document.head.querySelector<HTMLMetaElement>(
    `meta[${attr}="${key}"]`,
  );
  if (!el) {
    el = document.createElement("meta");
    el.setAttribute(attr, key);
    document.head.appendChild(el);
  }
  el.setAttribute("content", content);
}

export function Seo({
  title,
  description = site.description,
  path,
  type = "website",
}: SeoProps) {
  useEffect(() => {
    document.title = title;
    upsertMeta("name", "description", description);
    upsertMeta("property", "og:title", title);
    upsertMeta("property", "og:description", description);
    upsertMeta("property", "og:type", type);
    upsertMeta("name", "twitter:title", title);
    upsertMeta("name", "twitter:description", description);
    if (site.siteUrl) {
      upsertMeta("property", "og:url", `${site.siteUrl}${path}`);
      upsertMeta("property", "og:image", `${site.siteUrl}/og.png`);
      upsertMeta("name", "twitter:image", `${site.siteUrl}/og.png`);
    }
  }, [title, description, path, type]);

  return null;
}
