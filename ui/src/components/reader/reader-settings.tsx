import { Slider } from '@/components/ui/slider';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import { useAppStore } from '@/stores/app-store';
import { cn } from '@/lib/utils';

export function ReaderSettings() {
  const { readerSettings, updateReaderSettings } = useAppStore();

  const themes = [
    { id: 'light' as const, label: 'Light', bg: 'bg-white', text: 'text-gray-900', border: 'border-gray-300' },
    { id: 'sepia' as const, label: 'Sepia', bg: 'bg-[#f4ecd8]', text: 'text-[#5b4636]', border: 'border-[#d4c5a9]' },
    { id: 'dark' as const, label: 'Dark', bg: 'bg-[#1a1a1a]', text: 'text-white', border: 'border-gray-600' },
  ];

  const fonts = [
    { id: 'Georgia, serif', label: 'Serif' },
    { id: '-apple-system, sans-serif', label: 'Sans' },
    { id: 'ui-monospace, monospace', label: 'Mono' },
  ];

  return (
    <div className="space-y-6 mt-6">
      <div className="space-y-3">
        <label className="text-sm font-medium">Font Size</label>
        <div className="flex items-center gap-3">
          <span className="text-xs">A</span>
          <Slider
            value={[readerSettings.fontSize]}
            onValueChange={(v) => updateReaderSettings({ fontSize: Array.isArray(v) ? v[0] : v })}
            min={12}
            max={28}
            step={1}
            className="flex-1"
          />
          <span className="text-lg font-bold">A</span>
        </div>
        <p className="text-xs text-muted-foreground text-center">{readerSettings.fontSize}px</p>
      </div>

      <Separator />

      <div className="space-y-3">
        <label className="text-sm font-medium">Line Spacing</label>
        <Slider
          value={[readerSettings.lineHeight]}
          onValueChange={(v) => updateReaderSettings({ lineHeight: Array.isArray(v) ? v[0] : v })}
          min={1.2}
          max={2.4}
          step={0.1}
          className="flex-1"
        />
        <p className="text-xs text-muted-foreground text-center">{readerSettings.lineHeight.toFixed(1)}</p>
      </div>

      <Separator />

      <div className="space-y-3">
        <label className="text-sm font-medium">Theme</label>
        <div className="flex gap-2">
          {themes.map((t) => (
            <button
              key={t.id}
              onClick={() => updateReaderSettings({ theme: t.id })}
              className={cn(
                'flex-1 rounded-lg py-3 text-xs font-medium border-2 transition-colors',
                t.bg, t.text,
                readerSettings.theme === t.id ? 'border-primary' : t.border
              )}
            >
              {t.label}
            </button>
          ))}
        </div>
      </div>

      <Separator />

      <div className="space-y-3">
        <label className="text-sm font-medium">Font Family</label>
        <div className="flex gap-2">
          {fonts.map((f) => (
            <Button
              key={f.id}
              variant={readerSettings.fontFamily === f.id ? 'default' : 'outline'}
              size="sm"
              className="flex-1"
              onClick={() => updateReaderSettings({ fontFamily: f.id })}
            >
              {f.label}
            </Button>
          ))}
        </div>
      </div>
    </div>
  );
}
