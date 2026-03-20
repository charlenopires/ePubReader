import { BookCard } from './book-card';
import type { Book } from '@/lib/tauri';

interface BookGridProps {
  books: Book[];
  onRead: (bookId: string) => void;
  onTranslate: (bookId: string) => void;
  onDelete: (bookId: string) => void;
}

export function BookGrid({ books, onRead, onTranslate, onDelete }: BookGridProps) {
  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
      {books.map((book) => (
        <BookCard key={book.id} book={book} onRead={onRead} onTranslate={onTranslate} onDelete={onDelete} />
      ))}
    </div>
  );
}
