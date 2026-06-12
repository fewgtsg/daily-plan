import { useEffect, useState } from 'react';
import { useSearchParams, Link } from 'react-router-dom';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { api } from '@/lib/api';
import type { Entry, Task, Tag, SearchResult } from '@/types';

export function SearchPage() {
  const [searchParams] = useSearchParams();
  const tagParam = searchParams.get('tag');

  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [searched, setSearched] = useState(false);

  const [tagResults, setTagResults] = useState<{ entries: Entry[]; tasks: Task[] } | null>(null);
  const [allTags, setAllTags] = useState<Tag[]>([]);

  useEffect(() => {
    if (!tagParam) {
      setTagResults(null);
      return;
    }
    api.searchByTag(tagParam).then((res) => {
      if (res) setTagResults(res);
    });
  }, [tagParam]);

  useEffect(() => {
    api.getAllTags().then((tags) => tags && setAllTags(tags));
  }, []);

  const handleSearch = () => {
    if (!query.trim()) return;
    api.search(query.trim()).then((res) => {
      if (res) {
        setResults(res);
        setSearched(true);
      }
    }).catch((err) => console.error(err));
  };

  if (tagParam) {
    return (
      <div className="h-full flex flex-col">
        <h2 className="text-xl font-bold mb-4">
          标签 <Badge variant="secondary">#{tagParam}</Badge>
        </h2>
        <div className="flex-1 grid grid-cols-2 gap-4 overflow-auto">
          <section>
            <h3 className="text-sm font-semibold mb-2 text-muted-foreground">日记</h3>
            <div className="space-y-2">
              {tagResults?.entries.length === 0 && (
                <div className="text-sm text-muted-foreground">无相关日记</div>
              )}
              {tagResults?.entries.map((entry) => (
                <Link
                  key={entry.id}
                  to={`/?date=${entry.date}`}
                  className="block border rounded p-3 hover:bg-accent"
                >
                  <div className="text-sm font-medium">{entry.date}</div>
                  <div className="text-sm text-muted-foreground line-clamp-3" dangerouslySetInnerHTML={{ __html: entry.content }} />
                </Link>
              ))}
            </div>
          </section>
          <section>
            <h3 className="text-sm font-semibold mb-2 text-muted-foreground">任务</h3>
            <div className="space-y-2">
              {tagResults?.tasks.length === 0 && (
                <div className="text-sm text-muted-foreground">无相关任务</div>
              )}
              {tagResults?.tasks.map((task) => (
                <Link
                  key={task.id}
                  to={`/board?highlight=${encodeURIComponent(task.title)}`}
                  className="block border rounded p-3 hover:bg-accent"
                >
                  <div className="text-sm font-medium">{task.title}</div>
                  <div className="text-sm text-muted-foreground line-clamp-2">{task.description}</div>
                </Link>
              ))}
            </div>
          </section>
        </div>
      </div>
    );
  }

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
      <div className="flex flex-wrap gap-2 mb-4">
        {allTags.slice(0, 20).map((tag) => (
          <Link key={tag.id} to={`/search?tag=${encodeURIComponent(tag.name)}`}>
            <Badge variant="outline" className="cursor-pointer">
              {tag.displayName || tag.name} ({tag.usageCount})
            </Badge>
          </Link>
        ))}
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
