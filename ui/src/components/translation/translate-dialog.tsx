import { useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Progress } from '@/components/ui/progress';
import { useTranslation } from '@/hooks/use-translation';
import { useLmStudio } from '@/hooks/use-lmstudio';
import { useAppStore } from '@/stores/app-store';

interface TranslateDialogProps {
  bookId: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete: () => void;
}

export function TranslateDialog({ bookId, open, onOpenChange, onComplete }: TranslateDialogProps) {
  const { translating, progress, languages, startTranslation } = useTranslation();
  const { connected, availableModels, currentModel, setModel } = useLmStudio();
  const { targetLanguage, setTargetLanguage } = useAppStore();
  const [error, setError] = useState<string | null>(null);

  const handleTranslate = async () => {
    if (!bookId || !targetLanguage) return;
    setError(null);
    try {
      await startTranslation(bookId, targetLanguage);
      onComplete();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{translating ? 'Translating...' : 'Translate Book'}</DialogTitle>
        </DialogHeader>

        {translating && progress ? (
          <div className="space-y-4 py-4">
            <Progress value={progress.percentage} />
            <p className="text-sm text-muted-foreground text-center">
              Chapter {progress.translated_chapters}/{progress.total_chapters}: {progress.current_chapter_title}
            </p>
          </div>
        ) : (
          <div className="space-y-4 py-4">
            {!connected && (
              <p className="text-sm text-destructive">LM Studio is not running. Please start it first.</p>
            )}
            <div className="space-y-2">
              <label className="text-sm font-medium">Target Language</label>
              <Select value={targetLanguage} onValueChange={(v) => v && setTargetLanguage(v)}>
                <SelectTrigger>
                  <SelectValue placeholder="Select language" />
                </SelectTrigger>
                <SelectContent>
                  {languages.map((lang) => (
                    <SelectItem key={lang.code} value={lang.code}>
                      {lang.native_name} ({lang.name})
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Model</label>
              <Select value={currentModel ?? ''} onValueChange={(v) => v && setModel(v)}>
                <SelectTrigger>
                  <SelectValue placeholder="Select model" />
                </SelectTrigger>
                <SelectContent>
                  {availableModels.map((model) => (
                    <SelectItem key={model} value={model}>{model}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            {error && <p className="text-sm text-destructive">{error}</p>}
          </div>
        )}

        <DialogFooter>
          {!translating && (
            <>
              <Button variant="ghost" onClick={() => onOpenChange(false)}>Cancel</Button>
              <Button onClick={handleTranslate} disabled={!connected || !targetLanguage}>
                Start Translation
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
