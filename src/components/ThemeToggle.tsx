import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';

export function ThemeToggle() {
  const [theme, setTheme] = useState<'light' | 'dark'>('light');

  useEffect(() => {
    const root = window.document.documentElement;
    root.classList.remove('light', 'dark');
    root.classList.add(theme);
  }, [theme]);

  return (
    <Button variant="ghost" size="sm" onClick={() => setTheme((t) => (t === 'light' ? 'dark' : 'light'))}>
      {theme === 'light' ? '🌙' : '☀️'}
    </Button>
  );
}
