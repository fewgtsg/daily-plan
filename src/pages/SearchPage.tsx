import { useState } from 'react';
import { Link } from 'react-router-dom';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { api } from '@/lib/api';
import type { SearchResult } from '@/types';

export function SearchPage() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [searched, setSearched] = useState(false);

  const handleSearch = () => {
    if (!query.trim()) return;
    api.search(query.trim()).then((res) => {
      if (res) {
        setResults(res);
        setSearched(true);
      }
    }).catch((err) => console.error(err));
  };

  return (
    <div className="h-full flex flex-col">
      <h2 className="text-xl font-bold mb-4">历史搜索</h2>
      <div className="flex gap-2 mb-4">
        <Input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="搜索记录或任务..."
          onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
        />
        <Button onClick={handleSearch}>搜索</Button>
      </div>
      <div className="flex-1 overflow-auto space-y-2">
        {searched && results.length === 0 && (
          <div className="text-muted-foreground">未找到结果</div>
        )}
        {results.map((r) => (
          <div key={`${r.result_type}-${r.id}`} className="border rounded p-3 hover:bg-accent">
            <div className="flex items-center gap-2 mb-1">
              <span className="text-xs px-1.5 py-0.5 rounded bg-primary/10 text-primary">
                {r.result_type === 'entry' ? '记录' : '任务'}
              </span>
              {r.date && (
                <Link to={`/?date=${r.date}`} className="text-sm font-medium hover:underline">
                  {r.date}
                </Link>
              )}
            </div>
            <div className="text-sm text-muted-foreground line-clamp-3">{r.content}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
