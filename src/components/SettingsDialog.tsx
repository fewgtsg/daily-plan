import { useState } from 'react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';

export function SettingsDialog() {
  const [exporting, setExporting] = useState(false);

  const handleExport = async () => {
    const path = await save({ filters: [{ name: 'JSON', extensions: ['json'] }] });
    if (!path) return;
    setExporting(true);
    await invoke('export_all_data', { exportPath: path });
    setExporting(false);
    alert('导出成功');
  };

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="ghost" size="sm">设置</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>设置</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <div>
            <div className="text-sm font-medium mb-1">数据导出</div>
            <Button size="sm" onClick={handleExport} disabled={exporting}>
              {exporting ? '导出中...' : '导出全部数据为 JSON'}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
