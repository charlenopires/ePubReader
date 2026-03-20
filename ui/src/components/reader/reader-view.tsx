import { useEffect, useCallback } from 'react';
import { ChevronLeft, ChevronRight, ArrowLeft, List, Settings } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet';
import { useReader } from '@/hooks/use-reader';
import { useAppStore } from '@/stores/app-store';
import { ReaderSettings } from './reader-settings';
import { cn } from '@/lib/utils';

interface ReaderViewProps {
  bookId: string;
  onClose: () => void;
}

export function ReaderView({ bookId, onClose }: ReaderViewProps) {
  const { bookContent, chapter, content, currentChapter, loading, showTranslated, setShowTranslated, nextChapter, prevChapter, goToChapter, totalChapters } = useReader(bookId);
  const { readerSettings } = useAppStore();

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === 'ArrowLeft') prevChapter();
    if (e.key === 'ArrowRight') nextChapter();
    if (e.key === 'Escape') onClose();
  }, [prevChapter, nextChapter, onClose]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen bg-background">
        <div className="animate-spin rounded-full h-8 w-8 border-2 border-primary border-t-transparent" />
      </div>
    );
  }

  if (!bookContent || !chapter) return null;

  const progressPercent = totalChapters > 0 ? ((currentChapter + 1) / totalChapters) * 100 : 0;

  const readerBg = readerSettings.theme === 'sepia' ? 'bg-[#f4ecd8]' : readerSettings.theme === 'light' ? 'bg-white' : 'bg-background';
  const readerText = readerSettings.theme === 'sepia' ? 'text-[#5b4636]' : readerSettings.theme === 'light' ? 'text-gray-900' : 'text-foreground';

  return (
    <div className={cn('flex flex-col h-screen', readerBg, readerText)}>
      {/* Top toolbar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border/50 bg-background/80 backdrop-blur">
        <div className="flex items-center gap-2">
          <Button variant="ghost" size="icon" onClick={onClose}>
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <span className="text-sm font-medium truncate max-w-xs">{chapter.title}</span>
        </div>
        <div className="flex items-center gap-1">
          {chapter.translated_content && (
            <Button variant="ghost" size="sm" onClick={() => setShowTranslated(!showTranslated)} className="text-xs">
              {showTranslated ? 'Original' : 'Translated'}
            </Button>
          )}
          {/* TOC Sheet */}
          <Sheet>
            <SheetTrigger render={<Button variant="ghost" size="icon" />}>
              <List className="h-4 w-4" />
            </SheetTrigger>
            <SheetContent side="left">
              <SheetHeader><SheetTitle>Table of Contents</SheetTitle></SheetHeader>
              <ScrollArea className="h-[calc(100vh-80px)] mt-4">
                <div className="space-y-1">
                  {bookContent.chapters.map((ch, i) => (
                    <button
                      key={ch.id}
                      onClick={() => goToChapter(i)}
                      className={cn(
                        'w-full text-left px-3 py-2 rounded-md text-sm transition-colors',
                        i === currentChapter ? 'bg-accent text-accent-foreground font-medium' : 'hover:bg-accent/50 text-muted-foreground'
                      )}
                    >
                      {ch.title}
                    </button>
                  ))}
                </div>
              </ScrollArea>
            </SheetContent>
          </Sheet>
          {/* Settings Sheet */}
          <Sheet>
            <SheetTrigger render={<Button variant="ghost" size="icon" />}>
              <Settings className="h-4 w-4" />
            </SheetTrigger>
            <SheetContent>
              <SheetHeader><SheetTitle>Reader Settings</SheetTitle></SheetHeader>
              <ReaderSettings />
            </SheetContent>
          </Sheet>
        </div>
      </div>

      {/* Chapter content */}
      <ScrollArea className="flex-1">
        <article
          className="max-w-2xl mx-auto px-6 py-10 prose prose-sm dark:prose-invert"
          style={{
            fontSize: `${readerSettings.fontSize}px`,
            lineHeight: readerSettings.lineHeight,
            fontFamily: readerSettings.fontFamily,
          }}
          dangerouslySetInnerHTML={{ __html: content }}
        />
      </ScrollArea>

      {/* Bottom bar */}
      <div className="border-t border-border/50 px-4 py-2 bg-background/80 backdrop-blur">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="icon" onClick={prevChapter} disabled={currentChapter === 0}>
            <ChevronLeft className="h-4 w-4" />
          </Button>
          <div className="flex-1 space-y-1">
            <Progress value={progressPercent} className="h-1" />
            <p className="text-center text-xs text-muted-foreground">
              {currentChapter + 1} / {totalChapters}
            </p>
          </div>
          <Button variant="ghost" size="icon" onClick={nextChapter} disabled={currentChapter >= totalChapters - 1}>
            <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  );
}
