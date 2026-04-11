import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { TreeNode } from './types';
import { useAppStore } from './store/appStore';

export async function handleOpenFolder() {
  const store = useAppStore.getState();
  store.setStatusMessage('Opening folder dialog...');

  const selected = await open({
    directory: true,
    multiple: false,
    title: 'Select a folder containing PDF files',
  });

  if (!selected || Array.isArray(selected)) return;

  store.setRootPath(selected);
  store.setStatusMessage(`Scanning: ${selected}`);

  try {
    const result = await invoke<TreeNode | null>('scan_pdf_tree', { root: selected });
    if (result) {
      // The root node is the selected folder itself; use its children as the tree
      store.setTreeData(result.children ? result.children : [result]);
      store.setStatusMessage(
        `Loaded: ${selected} (${countPdfs(result)} PDF file(s) found)`
      );
    } else {
      store.setTreeData([]);
      store.setStatusMessage(`No PDF files found in: ${selected}`);
    }
  } catch (err) {
    store.setTreeData([]);
    store.setStatusMessage(`Error: ${err}`);
  }
}

function countPdfs(node: TreeNode): number {
  let count = node.isFolder ? 0 : 1;
  if (node.children) {
    for (const child of node.children) {
      count += countPdfs(child);
    }
  }
  return count;
}
