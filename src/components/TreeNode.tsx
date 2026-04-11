import { useState } from 'react';
import { TreeNode } from '../types';
import { useAppStore } from '../store/appStore';

interface TreeNodeProps {
  node: TreeNode;
  depth: number;
}

export function TreeNodeComponent({ node, depth }: TreeNodeProps) {
  const [expanded, setExpanded] = useState(false);
  const currentPdf = useAppStore((s) => s.currentPdf);
  const setCurrentPdf = useAppStore((s) => s.setCurrentPdf);

  const paddingLeft = depth * 16 + 8;

  if (node.isFolder) {
    const hasChildren = node.children && node.children.length > 0;
    return (
      <div>
        <div
          className="tree-item tree-folder"
          style={{ paddingLeft }}
          onClick={() => hasChildren && setExpanded(!expanded)}
        >
          <span className="tree-icon">
            {expanded ? '📂' : '📁'}
          </span>
          <span className="tree-name">{node.name}</span>
        </div>
        {expanded && hasChildren && (
          <div>
            {node.children!.map((child) => (
              <TreeNodeComponent key={child.path} node={child} depth={depth + 1} />
            ))}
          </div>
        )}
      </div>
    );
  }

  // PDF file
  const isActive = currentPdf === node.path;
  return (
    <div
      className={`tree-item tree-file ${isActive ? 'active' : ''}`}
      style={{ paddingLeft }}
      onClick={() => setCurrentPdf(node.path)}
      title={node.path}
    >
      <span className="tree-icon">📄</span>
      <span className="tree-name">{node.name}</span>
    </div>
  );
}
