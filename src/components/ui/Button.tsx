import type { ButtonHTMLAttributes, Ref } from "react";

type Variant = "primary" | "secondary" | "ghost" | "danger";

const variants: Record<Variant, string> = {
  primary:
    "bg-accent text-white hover:opacity-90 focus-visible:outline-accent disabled:hover:opacity-100",
  secondary:
    "border border-line bg-surface-2 text-ink hover:border-muted/50 disabled:hover:border-line",
  ghost: "text-muted hover:bg-surface-2 hover:text-ink",
  danger:
    "bg-critical/15 text-critical border border-critical/30 hover:bg-critical/25 disabled:hover:bg-critical/15",
};

const base =
  "inline-flex items-center justify-center gap-2 rounded-lg px-3.5 py-2 text-sm font-medium transition-colors motion-safe:active:scale-[0.98] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 disabled:pointer-events-none disabled:opacity-50";

export function Button({
  variant = "primary",
  className = "",
  type = "button",
  ref,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: Variant;
  ref?: Ref<HTMLButtonElement>;
}) {
  return (
    <button ref={ref} type={type} className={`${base} ${variants[variant]} ${className}`} {...props} />
  );
}
