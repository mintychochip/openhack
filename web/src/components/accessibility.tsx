import * as React from "react"
import { useEffect, useRef } from "react"

interface SkipLinkProps {
  targetId?: string;
  className?: string;
}

/**
 * Skip to main content link
 * Essential for keyboard navigation and screen reader users
 */
export function SkipLink({ targetId = "main-content", className }: SkipLinkProps) {
  return (
    <a
      href={`#${targetId}`}
      className={`
        sr-only
        focus:not-sr-only
        focus:absolute
        focus:top-4
        focus:left-4
        focus:z-50
        focus:px-4
        focus:py-2
        focus:bg-blue-600
        focus:text-white
        focus:rounded-md
        focus:outline-none
        focus:ring-2
        focus:ring-blue-500
        focus:ring-offset-2
        ${className || ''}
      `}
    >
      Skip to main content
    </a>
  )
}

interface LiveRegionProps {
  message: string;
  priority?: 'polite' | 'assertive';
  atomic?: boolean;
}

/**
 * ARIA live region for announcing dynamic content to screen readers
 */
export function LiveRegion({ message, priority = 'polite', atomic = true }: LiveRegionProps) {
  return (
    <div
      aria-live={priority}
      aria-atomic={atomic}
      className="sr-only"
    >
      {message}
    </div>
  )
}

interface VisuallyHiddenProps {
  children: React.ReactNode;
  as?: React.ElementType;
}

/**
 * Visually hidden component
 * Content is hidden visually but available to screen readers
 */
export function VisuallyHidden({ children, as = 'span' }: VisuallyHiddenProps) {
  const Component = as
  
  return (
    <Component
      style={{
        position: 'absolute',
        width: '1px',
        height: '1px',
        padding: '0',
        margin: '-1px',
        overflow: 'hidden',
        clip: 'rect(0, 0, 0, 0)',
        whiteSpace: 'nowrap',
        borderWidth: '0',
      }}
    >
      {children}
    </Component>
  )
}

interface FocusTrapProps {
  children: React.ReactNode;
  enabled?: boolean;
  onEscape?: () => void;
}

/**
 * Focus trap for modals and dialogs
 * Keeps focus within the container for accessibility
 */
export function FocusTrap({ children, enabled = true, onEscape }: FocusTrapProps) {
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!enabled) return

    const container = containerRef.current
    if (!container) return

    const focusableElements = container.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    )
    const firstElement = focusableElements[0]
    const lastElement = focusableElements[focusableElements.length - 1]

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onEscape?.()
        return
      }

      if (event.key !== 'Tab') return

      if (event.shiftKey) {
        if (document.activeElement === firstElement) {
          lastElement?.focus()
          event.preventDefault()
        }
      } else {
        if (document.activeElement === lastElement) {
          firstElement?.focus()
          event.preventDefault()
        }
      }
    }

    container.addEventListener('keydown', handleKeyDown)
    
    // Focus first element on mount
    firstElement?.focus()

    return () => {
      container.removeEventListener('keydown', handleKeyDown)
    }
  }, [enabled, onEscape])

  return (
    <div ref={containerRef}>
      {children}
    </div>
  )
}

interface IconProps {
  decorative?: boolean;
  title?: string;
}

/**
 * Get accessibility props for icon components
 */
export function getIconProps({ decorative = false, title }: IconProps) {
  if (decorative) {
    return {
      'aria-hidden': true,
      focusable: 'false',
    }
  }

  if (title) {
    return {
      'aria-label': title,
      role: 'img',
    }
  }

  return {}
}
