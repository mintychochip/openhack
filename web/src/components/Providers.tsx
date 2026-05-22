'use client';

import {AuthProvider} from '@/contexts/auth-context';
import {ThemeProvider} from '@/contexts/theme-context';
import {NextIntlClientProvider} from 'next-intl';
import {SSEProvider} from '@/components/sse-provider';

import type { ThemeConfig } from '@/contexts/theme-context';

type Props = {
  children: React.ReactNode;
  locale: string;
  messages: any;
  initialConfig?: ThemeConfig;
};

export function Providers({children, locale, messages, initialConfig}: Props) {
  return (
    <NextIntlClientProvider locale={locale} messages={messages}>
      <AuthProvider>
        <ThemeProvider initialConfig={initialConfig}>
          <SSEProvider>
            {children}
          </SSEProvider>
        </ThemeProvider>
      </AuthProvider>
    </NextIntlClientProvider>
  );
}
