"use client"

export function Badge({
  className = "",
  variant = "default",
  children,
  onClick,
}: {
  className?: string
  variant?: "default" | "secondary" | "destructive" | "outline"
  children: React.ReactNode
  onClick?: () => void
}) {
  const variants = {
    default: "bg-primary text-primary-foreground hover:bg-primary/80",
    secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
    destructive: "bg-destructive text-destructive-foreground hover:bg-destructive/80",
    outline: "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
  }

  return (
    <div
      onClick={onClick}
      className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold transition-colors ${variants[variant]} ${className}`}
    >
      {children}
    </div>
  )
}
