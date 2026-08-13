export function InfoRow({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="flex items-center justify-between gap-4 border-b border-line py-2.5 last:border-0">
      <dt className="shrink-0 text-sm text-muted">{label}</dt>
      <dd
        className={`min-w-0 truncate text-right text-sm font-medium text-ink ${
          mono ? "font-mono text-[13px]" : ""
        }`}
        title={value}
      >
        {value}
      </dd>
    </div>
  );
}

export function InfoList({ children }: { children: React.ReactNode }) {
  return <dl className="divide-y divide-line">{children}</dl>;
}
