import { ScrollArea } from '@/components/ui/scroll-area';
import { BookCard } from '@/components/library/book-card';
import { EmptyState } from '@/components/library/empty-state';
import type { Book } from '@/lib/tauri';

interface ReadingNowPageProps {
  books: Book[];
  onAddBook: () => void;
  onRead: (bookId: string) => void;
  onTranslate: (bookId: string) => void;
  onDelete: (bookId: string) => void;
}

export function ReadingNowPage({ books, onAddBook, onRead, onTranslate, onDelete }: ReadingNowPageProps) {
  const inProgress = books.filter((b) => b.progress > 0 && b.progress < 1);
  const recent = books.filter((b) => b.last_read).sort((a, b) => (b.last_read ?? '').localeCompare(a.last_read ?? '')).slice(0, 10);
  const notStarted = books.filter((b) => b.progress === 0);

  if (books.length === 0) {
    return <EmptyState onAddBook={onAddBook} />;
  }

  return (
    <ScrollArea className="flex-1">
      <div className="p-6 space-y-8">
        {inProgress.length > 0 && (
          <section>
            <h3 className="text-lg font-semibold mb-4">Currently Reading</h3>
            <div className="flex gap-4 overflow-x-auto pb-2">
              {inProgress.map((book) => (
                <div key={book.id} className="w-40 flex-shrink-0">
                  <BookCard book={book} onRead={onRead} onTranslate={onTranslate} onDelete={onDelete} />
                </div>
              ))}
            </div>
          </section>
        )}
        {recent.length > 0 && (
          <section>
            <h3 className="text-lg font-semibold mb-4">Recent</h3>
            <div className="flex gap-4 overflow-x-auto pb-2">
              {recent.map((book) => (
                <div key={book.id} className="w-40 flex-shrink-0">
                  <BookCard book={book} onRead={onRead} onTranslate={onTranslate} onDelete={onDelete} />
                </div>
              ))}
            </div>
          </section>
        )}
        {notStarted.length > 0 && (
          <section>
            <h3 className="text-lg font-semibold mb-4">Want to Read</h3>
            <div className="flex gap-4 overflow-x-auto pb-2">
              {notStarted.map((book) => (
                <div key={book.id} className="w-40 flex-shrink-0">
                  <BookCard book={book} onRead={onRead} onTranslate={onTranslate} onDelete={onDelete} />
                </div>
              ))}
            </div>
          </section>
        )}
      </div>
    </ScrollArea>
  );
}
