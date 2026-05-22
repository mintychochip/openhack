declare module "culori" {
  export function parse(color: string): any
  export function oklch(color: any): { l: number; c: number; h: number | undefined } | undefined
  export function hsl(color: any): { h: number | undefined; s: number; l: number } | undefined
  export function formatCss(color: any): string
}
