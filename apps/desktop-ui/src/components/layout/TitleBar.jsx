import { useState, useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, Square, X, Maximize2 } from 'lucide-react';

export function TitleBar() {
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    let unlisten = null;
    const init = async () => {
      try {
        const win = getCurrentWindow();
        setIsMaximized(await win.isMaximized());
        unlisten = await win.onResized(async () => {
          setIsMaximized(await win.isMaximized());
        });
      } catch (e) {
        console.error("Failed to init window listeners", e);
      }
    };
    init();
    return () => {
      if (unlisten) unlisten();
    }
  }, []);

  const minimize = () => getCurrentWindow()?.minimize();
  const toggleMaximize = async () => {
    const win = getCurrentWindow();
    if (!win) return;
    await win.toggleMaximize();
    setIsMaximized(await win.isMaximized());
  };
  const close = () => getCurrentWindow()?.close();

  return (
    <div className="h-8 bg-background flex items-center justify-between select-none z-[100] border-b border-border/50 shrink-0 w-full">
      {/* Drag Region - Takes up all available space to the left of buttons */}
      <div data-tauri-drag-region className="flex-1 h-full flex items-center px-4">
        <span className="text-xs font-medium text-muted-foreground pointer-events-none">WhytChat V1</span>
      </div>

      {/* Window Controls - No drag region here */}
      <div className="flex items-center h-full bg-background">
        <button
          onClick={minimize}
          className="h-full w-10 flex items-center justify-center hover:bg-surface text-muted-foreground hover:text-foreground transition-colors"
        >
          <Minus size={14} />
        </button>
        <button
          onClick={toggleMaximize}
          className="h-full w-10 flex items-center justify-center hover:bg-surface text-muted-foreground hover:text-foreground transition-colors"
        >
          {isMaximized ? <Square size={12} /> : <Maximize2 size={12} />}
        </button>
        <button
          onClick={close}
          className="h-full w-10 flex items-center justify-center hover:bg-destructive hover:text-destructive-foreground text-muted-foreground transition-colors"
        >
          <X size={14} />
        </button>
      </div>
    </div>
  );
}
