'use client';

import {useLocale} from 'next-intl';
import {useRouter} from 'next/navigation';
import {Select, SelectContent, SelectItem, SelectTrigger, SelectValue} from '@/components/ui/select';

const locales = [
  {code: 'en', name: 'English'},
  {code: 'es', name: 'Español'}
];

export function LanguageSwitcher() {
  const locale = useLocale();
  const router = useRouter();

  const handleValueChange = (value: string) => {
    const newPath = window.location.pathname.replace(`/${locale}`, `/${value}`);
    router.push(newPath);
  };

  return (
    <Select value={locale} onValueChange={handleValueChange}>
      <SelectTrigger className="w-[120px]">
        <SelectValue placeholder="Select language" />
      </SelectTrigger>
      <SelectContent>
        {locales.map((loc) => (
          <SelectItem key={loc.code} value={loc.code}>
            {loc.name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
