"use client"

export function Slider({
  className = "",
  value,
  onValueChange,
  max,
  step,
  ...props
}: Omit<React.InputHTMLAttributes<HTMLInputElement>, 'value' | 'onChange'> & {
  value?: number[]
  onValueChange?: (values: number[]) => void
  max?: number
  step?: number
}) {
  return (
    <input
      type="range"
      className={`w-full h-2 bg-secondary rounded-lg appearance-none cursor-pointer accent-primary ${className}`}
      value={value?.[0]}
      max={max}
      step={step}
      onChange={(e) => onValueChange?.([Number(e.target.value)])}
      {...props}
    />
  )
}
