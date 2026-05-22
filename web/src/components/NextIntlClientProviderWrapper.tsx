'use client';

import {NextIntlClientProvider} from 'next-intl';
import {useMessages} from 'next-intl';

type Props = {
  children: React.ReactNode;
  locale: string;
};

export function NextIntlClientProviderWrapper({children, locale}: Props) {
  const messages = useMessages();

  return (
    <NextIntlClientProvider locale={locale} messages={messages}>
      {children}
    </NextIntlClientProvider>
  );
}
