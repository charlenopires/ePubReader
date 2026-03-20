import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';

// Types
export interface Book {
  id: string;
  title: string;
  author: string;
  cover_path: string | null;
  original_path: string;
  language: string;
  target_language: string | null;
  translation_status: 'NotStarted' | 'InProgress' | 'Completed' | 'Failed';
  added_at: string;
  last_read: string | null;
  progress: number;
}

export interface Chapter {
  id: string;
  book_id: string;
  title: string;
  content: string;
  translated_content: string | null;
  order: number;
  images: ImageReference[];
}

export interface ImageReference {
  id: string;
  original_path: string;
  local_path: string;
  alt_text: string | null;
  position: ImagePosition;
}

export interface ImagePosition {
  chapter_id: string;
  paragraph_index: number;
  position_type: 'Before' | 'After' | 'Inline';
}

export interface BookContent {
  book: Book;
  chapters: Chapter[];
  current_chapter: number;
}

export interface LmStudioStatus {
  is_running: boolean;
  available_models: string[];
  recommended_model: string | null;
}

export interface Language {
  code: string;
  name: string;
  native_name: string;
}

export interface TranslationProgress {
  book_id: string;
  total_chapters: number;
  translated_chapters: number;
  current_chapter_title: string;
  percentage: number;
}

export const api = {
  checkLmStudioStatus: () => invoke<LmStudioStatus>('check_lmstudio_status'),
  getAvailableModels: () => invoke<string[]>('get_available_models'),
  setModel: (model: string) => invoke('set_lmstudio_model', { model }),
  getCurrentModel: () => invoke<string>('get_current_model'),
  getBooks: () => invoke<Book[]>('get_books'),
  addBook: (filePath: string) => invoke<Book>('add_book', { filePath }),
  getBookContent: (bookId: string) => invoke<BookContent>('get_book_content', { bookId }),
  deleteBook: (bookId: string) => invoke('delete_book', { bookId }),
  translateBook: (bookId: string, targetLanguage: string) => invoke('translate_book', { bookId, targetLanguage }),
  getSupportedLanguages: () => invoke<Language[]>('get_supported_languages'),
  generateBookHtml: (bookId: string) => invoke<string>('generate_book_html', { bookId }),
  onTranslationProgress: (callback: (progress: TranslationProgress) => void) =>
    listen<TranslationProgress>('translation-progress', (event) => callback(event.payload)),
  pickEpubFile: () => open({
    filters: [{ name: 'ePub', extensions: ['epub'] }],
    multiple: false,
  }),
};
