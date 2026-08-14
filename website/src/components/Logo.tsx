export function Logo({ className = "h-8 w-8" }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 32 32"
      className={className}
      aria-hidden="true"
      focusable="false"
    >
      <rect width="32" height="32" rx="7" fill="#0a0f1a" />
      <path
        d="M16 5l9 3.5v6c0 5.5-3.6 9.6-9 11.5-5.4-1.9-9-6-9-11.5v-6L16 5z"
        fill="none"
        stroke="#38bdf8"
        strokeWidth="2"
        strokeLinejoin="round"
      />
      <path
        d="M12 16.5l2.7 2.7L20.5 13"
        fill="none"
        stroke="#34d399"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
