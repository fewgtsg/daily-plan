import { useEffect, useState, useCallback, useRef } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';
import { format, addDays, subDays } from 'date-fns';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { Button } from '@/components/ui/button';
import { api } from '@/lib/api';
import { defaultTemplates } from '@/lib/templates';
import { TaskPanel } from '@/components/TaskPanel';
import { TagManager } from '@/components/TagManager';
import { TaskLinkAutocomplete } from '@/components/TaskLinkAutocomplete';
import { TaskLinkMark } from '@/editor/taskLinkMark';
import type { Task } from '@/types';
import type { Editor } from '@tiptap/core';

export function RecordPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const navigate = useNavigate();
  const dateParam = searchParams.get('date') || format(new Date(), 'yyyy-MM-dd');
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(false);
  const [linkQuery, setLinkQuery] = useState('');
  const [linkPos, setLinkPos] = useState<{ top: number; left: number } | null>(null);
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const requestedDateRef = useRef(dateParam);

  const detectTaskLink = useCallback((editorInstance: Editor) => {
    const { from } = editorInstance.state.selection;
    const textBefore = editorInstance.getText().slice(0, from);
    const match = textBefore.match(/\[\[([^\]]*)$/);
    if (match) {
      const query = match[1];
      const coords = editorInstance.view.coordsAtPos(from);
      setLinkQuery(query);
      setLinkPos({
        top: coords.bottom + window.scrollY,
        left: coords.left + window.scrollX,
      });
    } else {
      setLinkQuery('');
      setLinkPos(null);
    }
  }, []);

  const debouncedSave = useCallback(
    (content: string) => {
      clearTimeout(saveTimerRef.current);
      saveTimerRef.current = setTimeout(() => {
        setSaving(true);
        api
          .saveEntry(dateParam, content)
          .then(() => {
            setSaving(false);
            api.syncEntryTags(dateParam, content);
            api.syncEntryTaskLinks(dateParam, content);
          })
          .catch(() => setSaving(false));
      }, 1000);
    },
    [dateParam]
  );

  const editor = useEditor({
    extensions: [
      StarterKit,
      Placeholder.configure({ placeholder: '开始记录今天的计划与复盘...' }),
      TaskLinkMark,
    ],
    content: '',
    onUpdate: ({ editor }) => {
      detectTaskLink(editor);
      debouncedSave(editor.getHTML());
    },
    onSelectionUpdate: ({ editor }) => {
      detectTaskLink(editor);
    },
  });

  useEffect(() => {
    return () => clearTimeout(saveTimerRef.current);
  }, []);

  useEffect(() => {
    setLoading(true);
    requestedDateRef.current = dateParam;
    api
      .getEntry(dateParam)
      .then((e) => {
        if (requestedDateRef.current === dateParam && editor) {
          editor.commands.setContent(e?.content || '');
        }
      })
      .finally(() => setLoading(false));
  }, [dateParam, editor]);

  const insertTemplate = (content: string) => {
    editor?.chain().focus().insertContent(content).run();
  };

  const insertTask = (task: Task) => {
    const text = `> **任务**: ${task.title}\n> ${task.description}\n\n`;
    editor?.chain().focus().insertContent(text).run();
  };

  const goDay = (offset: number) => {
    const current = new Date(dateParam + 'T00:00:00');
    const next = offset > 0 ? addDays(current, offset) : subDays(current, Math.abs(offset));
    setSearchParams({ date: format(next, 'yyyy-MM-dd') });
  };

  const handleEditorClick = (e: React.MouseEvent) => {
    const target = (e.target as HTMLElement).closest('[data-task-link]') as HTMLElement | null;
    if (target) {
      const raw = target.textContent || '';
      const title = raw.replace(/^\[\[|\]\]$/g, '');
      if (title) {
        navigate('/board?highlight=' + encodeURIComponent(title));
      }
    }
  };

  return (
    <div className="flex h-full">
      <div className="flex-1 flex flex-col min-w-0">
        {loading && <div className="text-sm text-muted-foreground mb-2">加载中...</div>}
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={() => goDay(-1)}>前一天</Button>
            <h2 className="text-xl font-bold">{dateParam}</h2>
            <Button variant="outline" size="sm" onClick={() => goDay(1)}>后一天</Button>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">{saving ? '保存中...' : '已自动保存'}</span>
            <select
              className="text-sm border rounded px-2 py-1 bg-background"
              onChange={(e) => {
                const tpl = defaultTemplates.find((t) => t.id === e.target.value);
                if (tpl) insertTemplate(tpl.content);
                e.target.value = '';
              }}
            >
              <option value="">插入模板</option>
              {defaultTemplates.map((t) => (
                <option key={t.id} value={t.id}>{t.label}</option>
              ))}
            </select>
          </div>
        </div>
        <div
          className="flex-1 border rounded-lg p-4 bg-card relative"
          onClick={handleEditorClick}
        >
          <EditorContent editor={editor} className="prose dark:prose-invert max-w-none h-full outline-none" />
          {linkPos && (
            <div style={{ position: 'fixed', top: linkPos.top, left: linkPos.left }}>
              <TaskLinkAutocomplete
                query={linkQuery}
                onSelect={(title) => {
                  if (!editor) return;
                  const { from } = editor.state.selection;
                  const textBefore = editor.getText().slice(0, from);
                  const match = textBefore.match(/\[\[([^\]]*)$/);
                  if (match) {
                    const queryStart = from - match[0].length;
                    editor
                      .chain()
                      .focus()
                      .deleteRange({ from: queryStart, to: from })
                      .insertContent(`[[${title}]]`)
                      .run();
                  }
                  setLinkPos(null);
                }}
                onClose={() => setLinkPos(null)}
              />
            </div>
          )}
        </div>
      </div>
      <div className="w-72 flex flex-col gap-4 ml-4 overflow-y-auto">
        <TaskPanel onInsertTask={insertTask} />
        <TagManager date={dateParam} />
      </div>
    </div>
  );
}
