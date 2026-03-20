import { BookOpen, Languages, Trash2 } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import type { Book } from '@/lib/tauri';

interface BookCardProps {
  book: Book;
  onRead: (bookId: string) => void;
  onTranslate: (bookId: string) => void;
  onDelete: (bookId: string) => void;
}

const statusColors: Record<string, string> = {
  NotStarted: 'bg-muted text-muted-foreground',
  InProgress: 'bg-yellow-500/20 text-yellow-400',
  Completed: 'bg-green-500/20 text-green-400',
  Failed: 'bg-red-500/20 text-red-400',
};

const statusLabels: Record<string, string> = {
  NotStarted: 'Not Translated',
  InProgress: 'Translating...',
  Completed: 'Translated',
  Failed: 'Failed',
};

export function BookCard({ book, onRead, onTranslate, onDelete }: BookCardProps) {
  return (
    <Card className="group relative overflow-hidden border-0 bg-card hover:shadow-xl transition-all duration-300 hover:-translate-y-1 cursor-pointer">
      <div className="aspect-[2/3] relative bg-muted overflow-hidden" onClick={() => onRead(book.id)}>
        {book.cover_path ? (
          <img
            src={`asset://localhost/${book.cover_path}`}
            alt={book.title}
            className="w-full h-full object-cover"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center bg-gradient-to-br from-primary/20 to-primary/5">
            <BookOpen className="h-12 w-12 text-muted-foreground/50" />
          </div>
        )}
        {book.translation_status !== 'NotStarted' && (
          <Badge className={cn('absolute top-2 right-2 text-[10px]', statusColors[book.translation_status])}>
            {statusLabels[book.translation_status]}
          </Badge>
        )}
        {book.progress > 0 && (
          <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/30">
            <div className="h-full bg-primary" style={{ width: `${book.progress * 100}%` }} />
          </div>
        )}
        <div className="absolute inset-0 bg-black/0 group-hover:bg-black/60 transition-colors flex items-center justify-center gap-2 opacity-0 group-hover:opacity-100">
          <Button size="sm" variant="secondary" onClick={(e) => { e.stopPropagation(); onRead(book.id); }}>
            <BookOpen className="h-3.5 w-3.5 mr-1" /> Read
          </Button>
          <Button size="sm" variant="secondary" onClick={(e) => { e.stopPropagation(); onTranslate(book.id); }}>
            <Languages className="h-3.5 w-3.5 mr-1" /> Translate
          </Button>
        </div>
      </div>
      <div className="p-3">
        <h3 className="font-medium text-sm leading-tight line-clamp-2">{book.title}</h3>
        <p className="text-xs text-muted-foreground mt-1 truncate">{book.author}</p>
      </div>
      <Button
        variant="ghost"
        size="icon"
        className="absolute top-2 left-2 h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity bg-black/50 hover:bg-red-500/80 text-white"
        onClick={(e) => { e.stopPropagation(); onDelete(book.id); }}
      >
        <Trash2 className="h-3 w-3" />
      </Button>
    </Card>
  );
}
