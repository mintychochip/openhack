"use client"

export function Switch({
  className = "",
  checked,
  onCheckedChange,
  id,
}: {
  className?: string
  checked: boolean
  onCheckedChange?: (checked: boolean) => void
  id?: string
}) {
  return (
    <button
      id={id}
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onCheckedChange?.(!checked)}
      className={`
        peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent
        transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2
        focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50
        ${checked ? "bg-primary" : "bg-input"}
        ${className}
      `}
    >
      <span
        className={`
          pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0
          transition-transform
          ${checked ? "translate-x-5" : "translate-x-0"}
        `}
      />
    </button>
  )
}
