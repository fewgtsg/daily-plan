import { useEffect, useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';
import { MiniCalendar } from './MiniCalendar';
import { api } from '@/lib/api';
import { PanelLeft } from 'lucide-react';

const COLLAPSED_KEY = 'sidebar_collapsed';

export function Sidebar() {
  const location = useLocation();
  const [collapsed, setCollapsed] = useState(false);

  useEffect(() => {
    api.getAppSetting(COLLAPSED_KEY).then((value) => {
      if (value !== null) {
        setCollapsed(value === 'true');
      }
    });
  }, []);

  const toggle = () => {
    const next = !collapsed;
    setCollapsed(next);
    api.setAppSetting(COLLAPSED_KEY, String(next));
  };

  const nav = [
    { path: '/', label: '今日记录', icon: '📝' },
    { path: '/board', label: '四象限看板', icon: '📊' },
    { path: '/search', label: '历史搜索', icon: '🔍' },
  ];

  return (
    <aside className={`border-r flex flex-col h-full bg-muted/40 transition-all ${collapsed ? 'w-16' : 'w-64'}`}>
      <div className="p-4 font-bold text-lg flex items-center justify-between">
        {!collapsed && <span>每日计划</span>}
      </div>
      <ScrollArea className="flex-1 px-3">
        <nav className="space-y-1 mb-6">
          {nav.map((item) => (
            <Link key={item.path} to={item.path} title={item.label}>
              <Button
                variant={location.pathname === item.path ? 'secondary' : 'ghost'}
                className={`${collapsed ? 'w-full px-2 justify-center' : 'w-full justify-start'}`}
              >
                <span className="text-lg">{item.icon}</span>
                {!collapsed && <span className="ml-2">{item.label}</span>}
              </Button>
            </Link>
          ))}
        </nav>
        {!collapsed && <MiniCalendar />}
      </ScrollArea>
      <div className="p-2 border-t">
        <Button variant="ghost" size="sm" className="w-full" onClick={toggle}>
          <PanelLeft className="h-4 w-4" />
          {!collapsed && <span className="ml-2">收起侧边栏</span>}
        </Button>
      </div>
    </aside>
  );
}
