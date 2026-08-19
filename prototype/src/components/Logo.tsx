type LogoProps = {
  size?: number;
};

export function Logo({ size = 22 }: LogoProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      fill="none"
      aria-hidden="true"
    >
      <rect
        x="1"
        y="1"
        width="30"
        height="30"
        rx="8"
        fill="var(--primary)"
      />
      <rect
        x="1"
        y="1"
        width="30"
        height="30"
        rx="8"
        fill="white"
        opacity="0.08"
      />
      <path
        d="M9 22.5L9 9.5L13 9.5L19 18.5L19 9.5L23 9.5L23 22.5L19 22.5L13 13.5L13 22.5Z"
        fill="white"
      />
      <rect
        x="22"
        y="6"
        width="1.6"
        height="2.4"
        rx="0.8"
        fill="var(--primary)"
        className="motion-safe:animate-[nerd-cursor-blink_1.1s_steps(1)_infinite]"
      />
    </svg>
  );
}
