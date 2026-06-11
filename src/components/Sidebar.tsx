import { Link, useLocation } from 'react-router-dom';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';
import { MiniCalendar } from './MiniCalendar';

export function Sidebar() {
  const location = useLocation();
  const nav = [
    { path: '/', label: '今日记录' },
    { path: '/board', label: '四象限看板' },
    { path: '/search', label: '历史搜索' },
  ];

  return (
    <aside className="w-64 border-r flex flex-col h-full bg-muted/40">
      <div className="p-4 font-bold text-lg">每日计划</div>
      <ScrollArea className="flex-1 px-3">
        <nav className="space-y-1 mb-6">
          {nav.map((item) => (
            <Link key={item.path} to={item.path}>
              <Button
                variant={location.pathname === item.path ? 'secondary' : 'ghost'}
                className="w-full justify-start"
              >
                {item.label}
              </Button>
            </Link>
          ))}
        </nav>
        <MiniCalendar />
      </ScrollArea>
    </aside>
  );
}
