import { create } from 'zustand';
import { TreeNode } from '../types';

interface AppState {
  rootPath: string | null;
  treeData: TreeNode[];
  currentPdf: string | null;
  statusMessage: string;
  sidebarWidth: number;

  setRootPath: (path: string) => void;
  setTreeData: (data: TreeNode[]) => void;
  setCurrentPdf: (path: string | null) => void;
  setStatusMessage: (msg: string) => void;
  setSidebarWidth: (width: number) => void;
  reset: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  rootPath: null,
  treeData: [],
  currentPdf: null,
  statusMessage: 'Ready',
  sidebarWidth: 280,

  setRootPath: (path) => set({ rootPath: path }),
  setTreeData: (data) => set({ treeData: data }),
  setCurrentPdf: (path) =>
    set({
      currentPdf: path,
      statusMessage: path ? `Viewing: ${path.split(/[\\/]/).pop()}` : 'No file selected',
    }),
  setStatusMessage: (msg) => set({ statusMessage: msg }),
  setSidebarWidth: (width) => set({ sidebarWidth: width }),
  reset: () =>
    set({
      rootPath: null,
      treeData: [],
      currentPdf: null,
      statusMessage: 'Ready',
    }),
}));
