import { ScrollArea } from '@/components/ui/scroll-area';
import { BookGrid } from '@/components/library/book-grid';
import { EmptyState } from '@/components/library/empty-state';
import type { Book } from '@/lib/tauri';

interface LibraryPageProps {
  books: Book[];
  loading: boolean;
  onAddBook: () => void;
  onRead: (bookId: string) => void;
  onTranslate: (bookId: string) => void;
  onDelete: (bookId: string) => void;
}

export function LibraryPage({ books, loading, onAddBook, onRead, onTranslate, onDelete }: LibraryPageProps) {
  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-8 w-8 border-2 border-primary border-t-transparent" />
      </div>
    );
  }

  if (books.length === 0) {
    return <EmptyState onAddBook={onAddBook} />;
  }

  return (
    <ScrollArea className="flex-1">
      <div className="p-6">
        <BookGrid books={books} onRead={onRead} onTranslate={onTranslate} onDelete={onDelete} />
      </div>
    </ScrollArea>
  );
}
