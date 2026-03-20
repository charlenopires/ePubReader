import { useState, useEffect, useCallback } from 'react';
import { api, type TranslationProgress } from '@/lib/tauri';

export function useTranslation() {
  const [translating, setTranslating] = useState(false);
  const [progress, setProgress] = useState<TranslationProgress | null>(null);
  const [languages, setLanguages] = useState<{ code: string; name: string; native_name: string }[]>([]);

  useEffect(() => {
    api.getSupportedLanguages().then(setLanguages).catch(console.error);
  }, []);

  useEffect(() => {
    const unlisten = api.onTranslationProgress((p) => {
      setProgress(p);
      if (p.percentage >= 100) {
        setTimeout(() => {
          setTranslating(false);
          setProgress(null);
        }, 1500);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const startTranslation = useCallback(async (bookId: string, targetLanguage: string) => {
    setTranslating(true);
    setProgress({ book_id: bookId, total_chapters: 0, translated_chapters: 0, current_chapter_title: 'Preparing...', percentage: 0 });
    try {
      await api.translateBook(bookId, targetLanguage);
    } catch (err) {
      console.error('Translation failed:', err);
      setTranslating(false);
      setProgress(null);
      throw err;
    }
  }, []);

  return { translating, progress, languages, startTranslation };
}
