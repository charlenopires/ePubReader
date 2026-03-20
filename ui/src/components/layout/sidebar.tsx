import { BookOpen, Library, Cpu } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useAppStore } from '@/stores/app-store';
import { Separator } from '@/components/ui/separator';
import { Badge } from '@/components/ui/badge';

export function Sidebar() {
  const { currentView, setView, lmstudioConnected, sidebarOpen } = useAppStore();

  if (!sidebarOpen) return null;

  const navItems = [
    { id: 'library' as const, label: 'Library', icon: Library },
    { id: 'reading-now' as const, label: 'Reading Now', icon: BookOpen },
  ];

  return (
    <aside className="w-60 border-r border-border bg-card/50 flex flex-col h-full">
      <div className="p-4">
        <h1 className="text-lg font-semibold tracking-tight">ePub Reader</h1>
      </div>
      <Separator />
      <nav className="flex-1 p-2 space-y-1">
        {navItems.map((item) => (
          <button
            key={item.id}
            onClick={() => setView(item.id)}
            className={cn(
              'w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
              currentView === item.id
                ? 'bg-accent text-accent-foreground'
                : 'text-muted-foreground hover:text-foreground hover:bg-accent/50'
            )}
          >
            <item.icon className="h-4 w-4" />
            {item.label}
          </button>
        ))}
      </nav>
      <Separator />
      <div className="p-4">
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Cpu className="h-3.5 w-3.5" />
          <span>LM Studio</span>
          <Badge variant={lmstudioConnected ? 'default' : 'secondary'} className="ml-auto text-[10px] px-1.5 py-0">
            {lmstudioConnected ? 'Connected' : 'Offline'}
          </Badge>
        </div>
      </div>
    </aside>
  );
}
