import { useState, useEffect, useCallback } from 'react';
import { api, type BookContent } from '@/lib/tauri';

export function useReader(bookId: string | null) {
  const [bookContent, setBookContent] = useState<BookContent | null>(null);
  const [currentChapter, setCurrentChapter] = useState(0);
  const [loading, setLoading] = useState(false);
  const [showTranslated, setShowTranslated] = useState(true);

  useEffect(() => {
    if (!bookId) return;
    let cancelled = false;

    const load = async () => {
      setLoading(true);
      try {
        const content = await api.getBookContent(bookId);
        if (!cancelled) {
          setBookContent(content);
          setCurrentChapter(0);
        }
      } catch (err) {
        console.error('Failed to load book:', err);
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    load();
    return () => { cancelled = true; };
  }, [bookId]);

  const nextChapter = useCallback(() => {
    if (bookContent && currentChapter < bookContent.chapters.length - 1) {
      setCurrentChapter((prev) => prev + 1);
    }
  }, [bookContent, currentChapter]);

  const prevChapter = useCallback(() => {
    if (currentChapter > 0) {
      setCurrentChapter((prev) => prev - 1);
    }
  }, [currentChapter]);

  const goToChapter = useCallback((index: number) => {
    if (bookContent && index >= 0 && index < bookContent.chapters.length) {
      setCurrentChapter(index);
    }
  }, [bookContent]);

  const chapter = bookContent?.chapters[currentChapter] ?? null;
  const content = chapter
    ? (showTranslated && chapter.translated_content) || chapter.content
    : '';

  return {
    bookContent,
    chapter,
    content,
    currentChapter,
    loading,
    showTranslated,
    setShowTranslated,
    nextChapter,
    prevChapter,
    goToChapter,
    totalChapters: bookContent?.chapters.length ?? 0,
  };
}
