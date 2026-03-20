import { BookPlus } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface EmptyStateProps {
  onAddBook: () => void;
}

export function EmptyState({ onAddBook }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center h-full py-20">
      <div className="rounded-full bg-muted p-6 mb-6">
        <BookPlus className="h-10 w-10 text-muted-foreground" />
      </div>
      <h3 className="text-xl font-semibold mb-2">Your Library is Empty</h3>
      <p className="text-muted-foreground mb-6 text-sm">Add your first ePub book to get started</p>
      <Button onClick={onAddBook}>
        <BookPlus className="h-4 w-4 mr-2" />
        Add Your First Book
      </Button>
    </div>
  );
}
