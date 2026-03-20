import { useState, useEffect } from 'react';
import { TooltipProvider } from '@/components/ui/tooltip';
import { MainLayout } from '@/components/layout/main-layout';
import { Header } from '@/components/layout/header';
import { LibraryPage } from '@/pages/library';
import { ReadingNowPage } from '@/pages/reading-now';
import { ReaderView } from '@/components/reader/reader-view';
import { TranslateDialog } from '@/components/translation/translate-dialog';
import { useBooks } from '@/hooks/use-books';
import { useLmStudio } from '@/hooks/use-lmstudio';
import { useAppStore } from '@/stores/app-store';

export default function App() {
  const { currentView, currentBookId, openReader, closeReader, theme } = useAppStore();
  const { books, loading, addBook, deleteBook, refreshBooks } = useBooks();
  useLmStudio(); // keep the hook alive for status checking

  const [translateBookId, setTranslateBookId] = useState<string | null>(null);
  const [translateOpen, setTranslateOpen] = useState(false);

  // Apply dark mode class
  useEffect(() => {
    document.documentElement.classList.toggle('dark', theme === 'dark');
  }, [theme]);

  const handleRead = (bookId: string) => openReader(bookId);
  const handleTranslate = (bookId: string) => {
    setTranslateBookId(bookId);
    setTranslateOpen(true);
  };

  if (currentView === 'reader' && currentBookId) {
    return <ReaderView bookId={currentBookId} onClose={closeReader} />;
  }

  return (
    <TooltipProvider>
      <MainLayout>
        <Header onAddBook={addBook} />
        {currentView === 'library' && (
          <LibraryPage
            books={books}
            loading={loading}
            onAddBook={addBook}
            onRead={handleRead}
            onTranslate={handleTranslate}
            onDelete={deleteBook}
          />
        )}
        {currentView === 'reading-now' && (
          <ReadingNowPage
            books={books}
            onAddBook={addBook}
            onRead={handleRead}
            onTranslate={handleTranslate}
            onDelete={deleteBook}
          />
        )}
        <TranslateDialog
          bookId={translateBookId}
          open={translateOpen}
          onOpenChange={setTranslateOpen}
          onComplete={() => {
            setTranslateOpen(false);
            refreshBooks();
          }}
        />
      </MainLayout>
    </TooltipProvider>
  );
}
