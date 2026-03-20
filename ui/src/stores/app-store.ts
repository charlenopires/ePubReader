import { create } from 'zustand';

interface ReaderSettings {
  fontSize: number;
  lineHeight: number;
  fontFamily: string;
  theme: 'light' | 'dark' | 'sepia';
}

interface AppStore {
  // Navigation
  currentView: 'library' | 'reading-now' | 'reader';
  currentBookId: string | null;
  setView: (view: 'library' | 'reading-now' | 'reader') => void;
  openReader: (bookId: string) => void;
  closeReader: () => void;

  // LMStudio
  lmstudioConnected: boolean;
  currentModel: string | null;
  availableModels: string[];
  setLmStudioStatus: (connected: boolean, models: string[]) => void;
  setCurrentModel: (model: string | null) => void;

  // Settings
  targetLanguage: string;
  setTargetLanguage: (lang: string) => void;
  theme: 'light' | 'dark';
  toggleTheme: () => void;

  // Reader
  readerSettings: ReaderSettings;
  updateReaderSettings: (settings: Partial<ReaderSettings>) => void;

  // Sidebar
  sidebarOpen: boolean;
  setSidebarOpen: (open: boolean) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  currentView: 'library',
  currentBookId: null,
  setView: (view) => set({ currentView: view }),
  openReader: (bookId) => set({ currentView: 'reader', currentBookId: bookId }),
  closeReader: () => set({ currentView: 'library', currentBookId: null }),

  lmstudioConnected: false,
  currentModel: null,
  availableModels: [],
  setLmStudioStatus: (connected, models) => set({ lmstudioConnected: connected, availableModels: models }),
  setCurrentModel: (model) => set({ currentModel: model }),

  targetLanguage: 'pt',
  setTargetLanguage: (lang) => set({ targetLanguage: lang }),
  theme: 'dark',
  toggleTheme: () => set((state) => ({ theme: state.theme === 'dark' ? 'light' : 'dark' })),

  readerSettings: {
    fontSize: 18,
    lineHeight: 1.8,
    fontFamily: 'Georgia, serif',
    theme: 'dark',
  },
  updateReaderSettings: (settings) =>
    set((state) => ({ readerSettings: { ...state.readerSettings, ...settings } })),

  sidebarOpen: true,
  setSidebarOpen: (open) => set({ sidebarOpen: open }),
}));
