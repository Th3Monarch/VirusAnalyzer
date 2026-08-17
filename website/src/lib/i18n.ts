export type Lang = "en" | "es";

interface Dictionary {
  nav: {
    home: string;
    features: string;
    security: string;
    documentation: string;
    download: string;
    faq: string;
    about: string;
    githubAria: string;
    discordAria: string;
    openMenu: string;
    closeMenu: string;
  };
  footer: {
    tagline: string;
    product: string;
    community: string;
    github: string;
    releases: string;
    issues: string;
    discord: string;
    configHint: string;
    description: string;
    disclaimer: string;
    copyright: string;
  };
  seo: {
    home: { title: string; description: string };
    download: { title: string; description: string };
    features: { title: string; description: string };
    security: { title: string; description: string };
    documentation: { title: string; description: string };
    faq: { title: string; description: string };
    about: { title: string; description: string };
    changelog: { title: string; description: string };
    notFound: { title: string; description: string };
  };
  common: {
    none: string;
    loading: string;
    step: string;
    version: string;
    copy: string;
    copied: string;
    skipToContent: string;
  };
  changelog: {
    loading: string;
    unavailable: string;
    unavailableDesc: string;
    configHint: string;
    viewReleases: string;
    noReleases: string;
    prerelease: string;
    noNotes: string;
    viewOnGitHub: string;
    fullHistory: string;
    notConfigured: string;
  };
  download: {
    eyebrow: string;
    title: string;
    requirements: string;
    recommended: string;
    downloadInstaller: string;
    downloadPortable: string;
    sourceTitle: string;
    sourceDesc: string;
    verifyTitle: string;
    verifyDesc: string;
    checksumsMissing: string;
    releaseNotes: string;
    copySha256: string;
    loading: string;
    unavailableTitle: string;
    unavailableDesc: string;
    githubConfigHint: string;
  };
  documentation: {
    eyebrow: string;
    title: string;
    sectionsAria: string;
  };
  notFound: {
    code: string;
    title: string;
    description: string;
    backHome: string;
  };
  languageSwitcher: {
    label: string;
  };
}

export const translations: Record<Lang, Dictionary> = {
  en: {
    nav: {
      home: "Home",
      features: "Features",
      security: "Security",
      documentation: "Documentation",
      download: "Download",
      faq: "FAQ",
      about: "About",
      githubAria: "View source on GitHub",
      discordAria: "Join our Discord server",
      openMenu: "Open menu",
      closeMenu: "Close menu",
    },
    footer: {
      tagline: "Analyze. Understand. Protect.",
      product: "Product",
      community: "Community",
      github: "GitHub",
      releases: "Releases",
      issues: "Issues",
      discord: "Discord",
      configHint: "Configure “discordUrl” in src/site.config.json to add the Discord link.",
      description:
        "Cross-platform malware analysis and threat assessment tool (Windows, macOS, Linux). Static analysis, explainable heuristics and hash-only reputation checks.",
      disclaimer:
        "VirusAnalyzer is an open-source analysis and threat assessment tool. It is not a replacement for a professional endpoint security solution.",
      copyright: "© {year} {app}. Analyze. Understand. Protect.",
    },
    seo: {
      home: {
        title: "VirusAnalyzer — Cross-Platform Malware Analysis Tool",
        description:
          "Analyze suspicious files with static analysis, explainable heuristic scoring and optional VirusTotal reputation checks on Windows, macOS and Linux.",
      },
      download: {
        title: "Download — VirusAnalyzer",
        description:
          "Download VirusAnalyzer for Windows, macOS and Linux. Verify your download with SHA-256 checksums.",
      },
      features: {
        title: "Features — VirusAnalyzer",
        description:
          "Static analysis, threat assessment, hash analysis and optional VirusTotal reputation checks for Windows, macOS and Linux.",
      },
      security: {
        title: "Security — VirusAnalyzer",
        description:
          "How VirusAnalyzer handles privacy, execution, network access, terminal access and download verification.",
      },
      documentation: {
        title: "Documentation — VirusAnalyzer",
        description:
          "Documentation for VirusAnalyzer: installation, first analysis, threat scores, VirusTotal, quarantine and more.",
      },
      faq: {
        title: "FAQ — VirusAnalyzer",
        description:
          "Frequently asked questions about VirusAnalyzer: execution, privacy, VirusTotal, verification and supported platforms.",
      },
      about: {
        title: "About — VirusAnalyzer",
        description:
          "About VirusAnalyzer: a cross-platform tool for static malware analysis, built for learning, research and defensive security.",
      },
      changelog: {
        title: "Changelog — VirusAnalyzer",
        description: "Release history for VirusAnalyzer.",
      },
      notFound: {
        title: "Page not found — VirusAnalyzer",
        description: "The page you are looking for does not exist.",
      },
    },
    common: {
      none: "None",
      loading: "Loading…",
      step: "Step {n}",
      version: "Latest version",
      copy: "Copy",
      copied: "Copied",
      skipToContent: "Skip to content",
    },
    changelog: {
      loading: "Loading releases…",
      unavailable: "Unable to retrieve releases.",
      unavailableDesc:
        "The GitHub API did not respond or no releases have been published yet.",
      configHint:
        "Configure “githubOwner” and “githubRepository” in src/site.config.json.",
      viewReleases: "View releases on GitHub",
      noReleases: "No releases have been published yet.",
      prerelease: "Pre-release",
      noNotes: "No release notes provided.",
      viewOnGitHub: "View on GitHub",
      fullHistory: "Full history is available on {link}.",
      notConfigured:
        "Releases are published on GitHub once the repository is configured.",
    },
    download: {
      eyebrow: "Download",
      title: "{app} for Windows, macOS & Linux",
      requirements: "Windows 10+ / macOS 10.15+ / Linux (webkit2gtk)",
      recommended: "Recommended",
      downloadInstaller: "Download installer",
      downloadPortable: "Download portable",
      sourceTitle: "Source Code",
      sourceDesc:
        "Everything you see here is open source. Review the code, report issues and build it yourself.",
      verifyTitle: "Verify your download",
      verifyDesc:
        "SHA-256 allows you to verify that the downloaded file matches the published release. Compare the hash below with the one you compute locally:",
      checksumsMissing:
        "Checksum files are not available for this release yet.",
      releaseNotes: "Release notes",
      copySha256: "Copy SHA-256 for {name}",
      loading: "Checking the latest release on GitHub…",
      unavailableTitle: "Unable to retrieve the latest release.",
      unavailableDesc:
        "The GitHub API did not respond or no release has been published yet. Downloads are distributed exclusively through GitHub Releases.",
      githubConfigHint:
        "Configure “githubOwner” and “githubRepository” in src/site.config.json to publish download links.",
    },
    documentation: {
      eyebrow: "Documentation",
      title: "Documentation",
      sectionsAria: "Documentation sections",
    },
    notFound: {
      code: "404",
      title: "Page not found",
      description: "The page you are looking for does not exist or has been moved.",
      backHome: "Back to home",
    },
    languageSwitcher: {
      label: "Language",
    },
  },
  es: {
    nav: {
      home: "Inicio",
      features: "Características",
      security: "Seguridad",
      documentation: "Documentación",
      download: "Descargar",
      faq: "FAQ",
      about: "Acerca de",
      githubAria: "Ver el código en GitHub",
      discordAria: "Únete a nuestro servidor de Discord",
      openMenu: "Abrir menú",
      closeMenu: "Cerrar menú",
    },
    footer: {
      tagline: "Analiza. Entiende. Protege.",
      product: "Producto",
      community: "Comunidad",
      github: "GitHub",
      releases: "Versiones",
      issues: "Incidencias",
      discord: "Discord",
      configHint: "Configura “discordUrl” en src/site.config.json para añadir el enlace de Discord.",
      description:
        "Herramienta de análisis de malware y evaluación de amenazas multiplataforma (Windows, macOS, Linux). Análisis estático, heurísticas explicables y comprobaciones de reputación solo por hash.",
      disclaimer:
        "VirusAnalyzer es una herramienta open-source de análisis y evaluación de amenazas. No sustituye a una solución profesional de seguridad de endpoints.",
      copyright: "© {year} {app}. Analiza. Entiende. Protege.",
    },
    seo: {
      home: {
        title: "VirusAnalyzer — Herramienta de análisis de malware multiplataforma",
        description:
          "Analiza archivos sospechosos con análisis estático, puntuación heurística explicable y comprobaciones opcionales de reputación en VirusTotal para Windows, macOS y Linux.",
      },
      download: {
        title: "Descargar — VirusAnalyzer",
        description:
          "Descarga VirusAnalyzer para Windows, macOS y Linux. Verifica tu descarga con los checksums SHA-256.",
      },
      features: {
        title: "Características — VirusAnalyzer",
        description:
          "Análisis estático, evaluación de amenazas, análisis de hashes y comprobaciones opcionales de reputación en VirusTotal para Windows, macOS y Linux.",
      },
      security: {
        title: "Seguridad — VirusAnalyzer",
        description:
          "Cómo gestiona VirusAnalyzer la privacidad, la ejecución, el acceso a red, el acceso al terminal y la verificación de descargas.",
      },
      documentation: {
        title: "Documentación — VirusAnalyzer",
        description:
          "Documentación de VirusAnalyzer: instalación, primer análisis, puntuaciones de amenaza, VirusTotal, cuarentena y más.",
      },
      faq: {
        title: "FAQ — VirusAnalyzer",
        description:
          "Preguntas frecuentes sobre VirusAnalyzer: ejecución, privacidad, VirusTotal, verificación y plataformas compatibles.",
      },
      about: {
        title: "Acerca de — VirusAnalyzer",
        description:
          "Acerca de VirusAnalyzer: una herramienta multiplataforma de análisis estático de malware, creada para el aprendizaje, la investigación y la seguridad defensiva.",
      },
      changelog: {
        title: "Historial de cambios — VirusAnalyzer",
        description: "Historial de versiones de VirusAnalyzer.",
      },
      notFound: {
        title: "Página no encontrada — VirusAnalyzer",
        description: "La página que buscas no existe.",
      },
    },
    common: {
      none: "Ninguno",
      loading: "Cargando…",
      step: "Paso {n}",
      version: "Última versión",
      copy: "Copiar",
      copied: "Copiado",
      skipToContent: "Saltar al contenido",
    },
    changelog: {
      loading: "Cargando versiones…",
      unavailable: "No se pudieron recuperar las versiones.",
      unavailableDesc:
        "La API de GitHub no respondió o aún no se ha publicado ninguna versión.",
      configHint:
        "Configura “githubOwner” y “githubRepository” en src/site.config.json.",
      viewReleases: "Ver versiones en GitHub",
      noReleases: "Aún no se ha publicado ninguna versión.",
      prerelease: "Pre-versión",
      noNotes: "No se proporcionaron notas de versión.",
      viewOnGitHub: "Ver en GitHub",
      fullHistory: "El historial completo está disponible en {link}.",
      notConfigured:
        "Las versiones se publican en GitHub cuando el repositorio está configurado.",
    },
    download: {
      eyebrow: "Descargar",
      title: "{app} para Windows, macOS y Linux",
      requirements: "Windows 10+ / macOS 10.15+ / Linux (webkit2gtk)",
      recommended: "Recomendado",
      downloadInstaller: "Descargar instalador",
      downloadPortable: "Descargar portable",
      sourceTitle: "Código fuente",
      sourceDesc:
        "Todo lo que ves aquí es código abierto. Revisa el código, informa de incidencias y compílalo tú mismo.",
      verifyTitle: "Verifica tu descarga",
      verifyDesc:
        "SHA-256 te permite verificar que el archivo descargado coincide con la versión publicada. Compara el hash de abajo con el que calcules localmente:",
      checksumsMissing:
        "Los archivos de checksums no están disponibles todavía para esta versión.",
      releaseNotes: "Notas de la versión",
      copySha256: "Copiar SHA-256 de {name}",
      loading: "Comprobando la última versión en GitHub…",
      unavailableTitle: "No se pudo obtener la última versión.",
      unavailableDesc:
        "La API de GitHub no respondió o aún no se ha publicado ninguna versión. Las descargas se distribuyen exclusivamente a través de GitHub Releases.",
      githubConfigHint:
        "Configura “githubOwner” y “githubRepository” en src/site.config.json para publicar los enlaces de descarga.",
    },
    documentation: {
      eyebrow: "Documentación",
      title: "Documentación",
      sectionsAria: "Secciones de documentación",
    },
    notFound: {
      code: "404",
      title: "Página no encontrada",
      description: "La página que buscas no existe o ha sido movida.",
      backHome: "Volver al inicio",
    },
    languageSwitcher: {
      label: "Idioma",
    },
  },
};

function lookup(
  obj: Record<string, unknown>,
  path: string,
): string | undefined {
  return path
    .split(".")
    .reduce<unknown>((acc, part) => {
      if (acc === null || typeof acc !== "object") return undefined;
      return (acc as Record<string, unknown>)[part];
    }, obj) as string | undefined;
}

export function translate(lang: Lang, key: string): string {
  const value =
    lookup(translations[lang] as unknown as Record<string, unknown>, key) ??
    lookup(translations.en as unknown as Record<string, unknown>, key);
  return typeof value === "string" ? value : key;
}
