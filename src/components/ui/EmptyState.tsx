import type { ComponentType, ReactNode } from "react";

export function EmptyState({
  icon: Icon,
  title,
  description,
  children,
}: {
  icon: ComponentType<{ className?: string }>;
  title: string;
  description?: string;
  children?: ReactNode;
}) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 px-6 py-12 text-center">
      <div className="flex size-12 items-center justify-center rounded-xl border border-line bg-surface-2 text-muted">
        <Icon className="size-6" />
      </div>
      <div>
        <p className="text-sm font-semibold text-ink">{title}</p>
        {description ? <p className="mt-1 max-w-md text-xs text-muted">{description}</p> : null}
      </div>
      {children}
    </div>
  );
}

export function TodoTag({ label }: { label: string }) {
  return (
    <span className="inline-flex items-center rounded-full border border-warn/40 bg-warn/10 px-2 py-0.5 text-[11px] font-medium text-warn">
      {label}
    </span>
  );
}
