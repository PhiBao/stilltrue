export function Mark({ size = 26 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 64 64"
      aria-hidden="true"
      focusable="false"
    >
      <rect width="64" height="64" rx="13" fill="#171512" />
      <path
        d="M15.5 33.5 26 44 48.5 19.5"
        fill="none"
        stroke="#f4efe4"
        strokeWidth="7.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="49" cy="15.5" r="4.5" fill="#b93b1c" />
    </svg>
  );
}
