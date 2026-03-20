import { Plus, Sun, Moon, PanelLeft } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useAppStore } from '@/stores/app-store';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';

interface HeaderProps {
  onAddBook: () => void;
}

export function Header({ onAddBook }: HeaderProps) {
  const { currentView, theme, toggleTheme, sidebarOpen, setSidebarOpen } = useAppStore();

  const titles: Record<string, string> = {
    library: 'Library',
    'reading-now': 'Reading Now',
    reader: '',
  };

  if (currentView === 'reader') return null;

  return (
    <header className="h-14 border-b border-border flex items-center justify-between px-4 bg-card/50 backdrop-blur">
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="icon" onClick={() => setSidebarOpen(!sidebarOpen)}>
          <PanelLeft className="h-4 w-4" />
        </Button>
        <h2 className="text-lg font-semibold">{titles[currentView]}</h2>
      </div>
      <div className="flex items-center gap-2">
        <Tooltip>
          <TooltipTrigger render={<Button variant="ghost" size="icon" onClick={toggleTheme} />}>
            {theme === 'dark' ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
          </TooltipTrigger>
          <TooltipContent>Toggle theme</TooltipContent>
        </Tooltip>
        <Button onClick={onAddBook} size="sm">
          <Plus className="h-4 w-4 mr-1" />
          Add Book
        </Button>
      </div>
    </header>
  );
}
