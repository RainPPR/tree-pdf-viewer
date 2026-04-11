import { TreeNodeComponent } from './TreeNode';
import { useAppStore } from '../store/appStore';

export function TreeView() {
  const treeData = useAppStore((s) => s.treeData);
  const rootPath = useAppStore((s) => s.rootPath);

  if (!rootPath) {
    return (
      <div className="tree-view tree-empty">
        <div className="tree-placeholder">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
          <p>Open a folder to browse PDF files</p>
        </div>
      </div>
    );
  }

  if (treeData.length === 0) {
    return (
      <div className="tree-view tree-empty">
        <div className="tree-placeholder">
          <p>No PDF files found in this folder</p>
        </div>
      </div>
    );
  }

  return (
    <div className="tree-view">
      {treeData.map((node) => (
        <TreeNodeComponent key={node.path} node={node} depth={0} />
      ))}
    </div>
  );
}
