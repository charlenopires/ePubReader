import { useState, useEffect, useCallback } from 'react';
import { api, type Book } from '@/lib/tauri';

export function useBooks() {
  const [books, setBooks] = useState<Book[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadBooks = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.getBooks();
      setBooks(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadBooks();
  }, [loadBooks]);

  const addBook = useCallback(async () => {
    try {
      const selected = await api.pickEpubFile();
      if (selected) {
        setLoading(true);
        await api.addBook(selected as string);
        await loadBooks();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setLoading(false);
    }
  }, [loadBooks]);

  const deleteBook = useCallback(async (bookId: string) => {
    try {
      await api.deleteBook(bookId);
      await loadBooks();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [loadBooks]);

  return { books, loading, error, addBook, deleteBook, refreshBooks: loadBooks };
}
