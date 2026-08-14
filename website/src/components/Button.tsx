import type { ReactNode } from "react";
import { Link } from "react-router-dom";

type Variant = "primary" | "secondary" | "ghost";

const base =
  "inline-flex items-center justify-center gap-2 rounded-lg px-5 py-2.5 text-sm font-semibold transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 focus-visible:ring-offset-2 focus-visible:ring-offset-night";

const variants: Record<Variant, string> = {
  primary:
    "bg-sky-400 text-ink hover:bg-sky-300",
  secondary:
    "border border-line-2 bg-ink-2 text-zinc-100 hover:border-sky-500/60 hover:bg-ink-3",
  ghost: "text-zinc-300 hover:text-white",
};

interface ButtonProps {
  children: ReactNode;
  variant?: Variant;
  to?: string;
  href?: string;
  className?: string;
  ariaLabel?: string;
  external?: boolean;
}

export function Button({
  children,
  variant = "primary",
  to,
  href,
  className = "",
  ariaLabel,
  external = false,
}: ButtonProps) {
  const cls = `${base} ${variants[variant]} ${className}`;
  if (to) {
    return (
      <Link to={to} className={cls} aria-label={ariaLabel}>
        {children}
      </Link>
    );
  }
  return (
    <a
      href={href}
      className={cls}
      aria-label={ariaLabel}
      {...(external ? { target: "_blank", rel: "noopener noreferrer" } : {})}
    >
      {children}
    </a>
  );
}
