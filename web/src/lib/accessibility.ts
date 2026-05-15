/**
 * Accessibility utilities for OpenHack
 * 
 * Provides helper functions and components for WCAG 2.1 AA compliance
 */

/**
 * Announce message to screen readers
 * Uses aria-live region to announce dynamic content changes
 */
export function announceToScreenReader(message: string, priority: 'polite' | 'assertive' = 'polite') {
  const announcement = document.getElementById('aria-live-announcer');
  if (announcement) {
    announcement.setAttribute('aria-live', priority);
    announcement.textContent = message;
  } else {
    // Create announcer if it doesn't exist
    const announcer = document.createElement('div');
    announcer.id = 'aria-live-announcer';
    announcer.setAttribute('aria-live', priority);
    announcer.setAttribute('aria-atomic', 'true');
    announcer.className = 'sr-only';
    document.body.appendChild(announcer);
    announcer.textContent = message;
  }
}

/**
 * Trap focus within a container (for modals/dialogs)
 * Returns a cleanup function to remove the trap
 */
export function trapFocus(container: HTMLElement) {
  const focusableElements = container.querySelectorAll<HTMLElement>(
    'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
  );
  const firstElement = focusableElements[0];
  const lastElement = focusableElements[focusableElements.length - 1];

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key !== 'Tab') return;

    if (event.shiftKey) {
      if (document.activeElement === firstElement) {
        lastElement.focus();
        event.preventDefault();
      }
    } else {
      if (document.activeElement === lastElement) {
        firstElement.focus();
        event.preventDefault();
      }
    }
  }

  container.addEventListener('keydown', handleKeyDown);

  // Focus first element
  firstElement?.focus();

  // Return cleanup function
  return () => {
    container.removeEventListener('keydown', handleKeyDown);
  };
}

/**
 * Check if element is visible (for screen readers)
 */
export function isElementVisible(element: HTMLElement): boolean {
  const style = window.getComputedStyle(element);
  return (
    style.display !== 'none' &&
    style.visibility !== 'hidden' &&
    style.opacity !== '0'
  );
}

/**
 * Get all focusable elements within a container
 */
export function getFocusableElements(container: HTMLElement): HTMLElement[] {
  return Array.from(
    container.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    )
  ).filter(el => !el.hasAttribute('disabled') && isElementVisible(el));
}

/**
 * Handle keyboard navigation for custom components
 */
export function handleKeyboardNavigation(
  event: React.KeyboardEvent,
  actions: {
    onEnter?: () => void;
    onEscape?: () => void;
    onArrowUp?: () => void;
    onArrowDown?: () => void;
    onArrowLeft?: () => void;
    onArrowRight?: () => void;
    onSpace?: () => void;
  }
) {
  switch (event.key) {
    case 'Enter':
      actions.onEnter?.();
      break;
    case 'Escape':
      actions.onEscape?.();
      break;
    case 'ArrowUp':
      actions.onArrowUp?.();
      event.preventDefault();
      break;
    case 'ArrowDown':
      actions.onArrowDown?.();
      event.preventDefault();
      break;
    case 'ArrowLeft':
      actions.onArrowLeft?.();
      event.preventDefault();
      break;
    case 'ArrowRight':
      actions.onArrowRight?.();
      event.preventDefault();
      break;
    case ' ':
      actions.onSpace?.();
      event.preventDefault();
      break;
  }
}

/**
 * Generate accessible label for icon-only buttons
 */
export function getAccessibleLabel(
  label: string,
  iconOnly: boolean
): { 'aria-label'?: string; 'aria-hidden'?: boolean } {
  if (iconOnly) {
    return { 'aria-label': label };
  }
  return { 'aria-hidden': label ? undefined : true };
}

/**
 * Validate color contrast ratio (WCAG 2.1 AA requires 4.5:1 for normal text)
 * Returns true if contrast meets WCAG AA standards
 */
export function hasSufficientContrast(
  foreground: string,
  background: string,
  largeText: boolean = false
): boolean {
  // Simplified check - in production use a library like 'wcag-contrast'
  const getLuminance = (hex: string) => {
    const rgb = parseInt(hex.slice(1), 16);
    const r = (rgb >> 16) & 0xff;
    const g = (rgb >> 8) & 0xff;
    const b = (rgb >> 0) & 0xff;
    
    const a = [r, g, b].map(v => {
      v /= 255;
      return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
    });
    
    return a[0] * 0.2126 + a[1] * 0.7152 + a[2] * 0.0722;
  };

  const l1 = getLuminance(foreground);
  const l2 = getLuminance(background);
  const ratio = l1 > l2 ? (l1 + 0.05) / (l2 + 0.05) : (l2 + 0.05) / (l1 + 0.05);
  
  return largeText ? ratio >= 3 : ratio >= 4.5;
}

/**
 * Accessible dropdown/menu component props
 */
export interface DropdownAccessibilityProps {
  'aria-haspopup': 'menu' | 'listbox' | 'tree' | 'grid' | 'dialog';
  'aria-expanded': boolean;
  'aria-controls': string;
  'aria-label'?: string;
}

/**
 * Accessible dialog/modal component props
 */
export interface DialogAccessibilityProps {
  role: 'dialog';
  'aria-modal': 'true';
  'aria-labelledby': string;
  'aria-describedby'?: string;
}

/**
 * Accessible tab component props
 */
export interface TabAccessibilityProps {
  role: 'tab';
  'aria-selected': boolean;
  'aria-controls': string;
  id: string;
  tabIndex: number;
}

/**
 * Accessible tablist component props
 */
export interface TabListAccessibilityProps {
  role: 'tablist';
  'aria-label': string;
}

/**
 * Accessible tabpanel component props
 */
export interface TabPanelAccessibilityProps {
  role: 'tabpanel';
  'aria-labelledby': string;
  id: string;
  tabIndex: 0;
}
