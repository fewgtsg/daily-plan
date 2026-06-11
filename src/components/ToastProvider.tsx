import { useEffect } from 'react';
import { Toaster } from '@/components/ui/toaster';
import { useToast } from '@/hooks/use-toast';
import { setToastCallback } from '@/lib/api';

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const { toast } = useToast();
  useEffect(() => {
    setToastCallback((msg: string) => toast({ title: '操作失败', description: msg, variant: 'destructive' }));
  }, [toast]);
  return (
    <>
      {children}
      <Toaster />
    </>
  );
}
