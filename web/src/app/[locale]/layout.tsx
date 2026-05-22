import {getMessages} from 'next-intl/server';
import {notFound} from 'next/navigation';
import {Providers} from '@/components/Providers';
import type { ThemeConfig } from '@/contexts/theme-context';

type Props = {
  children: React.ReactNode;
  params: {locale: string};
};

const locales = ['en', 'es'];

export function generateStaticParams() {
  return locales.map((locale) => ({locale}));
}

async function fetchThemeConfig(): Promise<ThemeConfig | undefined> {
  try {
    const gatewayUrl = process.env.GATEWAY_INTERNAL_URL || 'http://gateway-svc:8000';
    const res = await fetch(`${gatewayUrl}/api/core/info`, { cache: 'no-store' });
    if (!res.ok) return undefined;
    const data = await res.json();
    return {
      name: data.name || 'OpenHack',
      tagline: data.tagline || 'Build something amazing',
      logoUrl: data.logo_url || null,
      daisyuiPreset: data.daisyui_theme_preset || 'light',
      daisyuiCustomTheme: data.daisyui_custom_theme || {},
      customCss: data.custom_css || '',
      fontConfig: data.font_config || {},
    };
  } catch {
    return undefined;
  }
}

export default async function LocaleLayout({children, params}: Props) {
  const {locale} = params;

  if (!locales.includes(locale)) {
    notFound();
  }

  const messages = await getMessages();
  const initialConfig = await fetchThemeConfig();

  const darkPresets = ['dark', 'night', 'dracula', 'black', 'luxury', 'business', 'coffee', 'dim', 'winter', 'sunset', 'synthwave'];
  const isDark = darkPresets.includes(initialConfig?.daisyuiPreset || '');

  return (
    <html lang={locale} data-theme={initialConfig?.daisyuiPreset || 'light'} className={isDark ? 'dark' : ''}>
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link rel="icon" href="/favicon.ico" />
        {initialConfig?.fontConfig?.display?.url && (
          <link rel="stylesheet" href={initialConfig.fontConfig.display.url} />
        )}
        {initialConfig?.fontConfig?.heading?.url && (
          <link rel="stylesheet" href={initialConfig.fontConfig.heading.url} />
        )}
        {initialConfig?.fontConfig?.body?.url && (
          <link rel="stylesheet" href={initialConfig.fontConfig.body.url} />
        )}
        <style>{`
          :root {
            --font-display: ${initialConfig?.fontConfig?.display?.family || 'ui-sans-serif, system-ui, sans-serif'};
            --font-heading: ${initialConfig?.fontConfig?.heading?.family || 'ui-sans-serif, system-ui, sans-serif'};
            --font-body: ${initialConfig?.fontConfig?.body?.family || 'ui-sans-serif, system-ui, sans-serif'};
          }
        `}</style>
      </head>
      <body>
        <Providers locale={locale} messages={messages} initialConfig={initialConfig}>
          {children}
        </Providers>
      </body>
    </html>
  );
}
