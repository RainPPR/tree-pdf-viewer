import { FolderOpen, Settings } from 'lucide-react';
import { handleOpenFolder } from '../commands';
import { useAppStore } from '../store/appStore';

interface ToolbarProps {
  onOpenSettings: () => void;
}

export function Toolbar({ onOpenSettings }: ToolbarProps) {
  const rootPath = useAppStore((s) => s.rootPath);

  return (
    <div className="toolbar">
      <button className="toolbar-btn" onClick={handleOpenFolder} title="Open Folder">
        <FolderOpen size={16} strokeWidth={2} />
        Open Folder
      </button>
      {rootPath && (
        <span className="toolbar-path" title={rootPath}>
          {rootPath}
        </span>
      )}
      <div className="toolbar-spacer" />
      <button className="toolbar-btn toolbar-icon-btn" onClick={onOpenSettings} title="Settings">
        <Settings size={16} strokeWidth={2} />
      </button>
    </div>
  );
}
