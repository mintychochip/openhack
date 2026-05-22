import {useTranslations as _useTranslations, useLocale as _useLocale} from 'next-intl';

/**
 * Hook for accessing translations in client components.
 * 
 * @example
 * ```tsx
 * const t = useTranslations('auth.login');
 * return <h1>{t('title')}</h1>; // "Sign In"
 * ```
 * 
 * @param namespace - The namespace to use (e.g., 'auth.login', 'dashboard')
 * @returns Translation function
 */
export function useTranslations(namespace?: string) {
  return _useTranslations(namespace);
}

/**
 * Hook for getting the current locale.
 * 
 * @example
 * ```tsx
 * const locale = useLocale();
 * return <div>Current: {locale}</div>;
 * ```
 */
export function useLocale() {
  return _useLocale();
}
