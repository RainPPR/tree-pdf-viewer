import { useAppStore } from '../store/appStore';
import { handleOpenFolder } from '../commands';

export function Toolbar() {
  const rootPath = useAppStore((s) => s.rootPath);

  return (
    <div className="toolbar">
      <button className="toolbar-btn" onClick={handleOpenFolder} title="Open Folder">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
        </svg>
        Open Folder
      </button>
      {rootPath && (
        <span className="toolbar-path" title={rootPath}>
          {rootPath}
        </span>
      )}
    </div>
  );
}
