import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { format, startOfMonth, endOfMonth, eachDayOfInterval, isSameMonth, isToday } from 'date-fns';
import { api } from '@/lib/api';

export function MiniCalendar() {
  const [currentMonth, setCurrentMonth] = useState(new Date());
  const [entryDates, setEntryDates] = useState<Set<string>>(new Set());

  useEffect(() => {
    api.getEntryDates().then((dates) => setEntryDates(new Set(dates)));
  }, []);

  const days = eachDayOfInterval({
    start: startOfMonth(currentMonth),
    end: endOfMonth(currentMonth),
  });

  return (
    <div className="p-2">
      <div className="flex justify-between items-center mb-2 text-sm font-medium">
        <button onClick={() => setCurrentMonth((d) => new Date(d.getFullYear(), d.getMonth() - 1))}>&lt;</button>
        <span>{format(currentMonth, 'yyyy年 MM月')}</span>
        <button onClick={() => setCurrentMonth((d) => new Date(d.getFullYear(), d.getMonth() + 1))}>&gt;</button>
      </div>
      <div className="grid grid-cols-7 gap-1 text-center text-xs">
        {['日', '一', '二', '三', '四', '五', '六'].map((d) => (
          <div key={d} className="text-muted-foreground">{d}</div>
        ))}
        {days.map((day) => {
          const dateStr = format(day, 'yyyy-MM-dd');
          const hasEntry = entryDates.has(dateStr);
          return (
            <Link key={dateStr} to={`/?date=${dateStr}`}>
              <div className="relative h-7 w-7 mx-auto flex items-center justify-center rounded-full cursor-pointer hover:bg-accent">
                <span className={isToday(day) ? 'bg-primary text-primary-foreground rounded-full w-6 h-6 flex items-center justify-center' : ''}>
                  {format(day, 'd')}
                </span>
                {hasEntry && <span className="absolute bottom-0 w-1 h-1 bg-green-500 rounded-full" />}
                {!isSameMonth(day, currentMonth) && <span className="absolute inset-0 text-muted-foreground/30">{format(day, 'd')}</span>}
              </div>
            </Link>
          );
        })}
      </div>
    </div>
  );
}
