import { Outlet } from 'react-router-dom';
import { Sidebar } from './Sidebar';
import { ThemeToggle } from './ThemeToggle';
import { SettingsDialog } from './SettingsDialog';

export function Layout() {
  return (
    <div className="flex h-screen w-screen bg-background text-foreground overflow-hidden">
      <Sidebar />
      <main className="flex-1 overflow-auto p-6 relative">
        <div className="absolute top-4 right-4 flex gap-2">
          <ThemeToggle />
          <SettingsDialog />
        </div>
        <Outlet />
      </main>
    </div>
  );
}
