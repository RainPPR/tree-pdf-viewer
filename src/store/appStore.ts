import { create } from 'zustand';
import { TreeNode, Tab, AppSettings, DEFAULT_SETTINGS } from '../types';

interface AppState {
  rootPath: string | null;
  treeData: TreeNode[];
  activeTabId: string | null;
  tabs: Tab[];
  settings: AppSettings;
  statusMessage: string;
  sidebarWidth: number;
  memoryUsageMB: number;
  memoryWarning: string | null;

  setRootPath: (path: string) => void;
  setTreeData: (data: TreeNode[]) => void;

  addTab: (pdfPath: string, title: string) => void;
  closeTab: (tabId: string) => void;
  setActiveTab: (tabId: string | null) => void;
  getTabIndex: (tabId: string) => number;

  updateSettings: (settings: Partial<AppSettings>) => void;
  setMemoryUsage: (mb: number) => void;
  setMemoryWarning: (msg: string | null) => void;

  setStatusMessage: (msg: string) => void;
  setSidebarWidth: (width: number) => void;
  reset: () => void;
}

let tabCounter = 0;
function generateTabId(): string {
  return `tab-${++tabCounter}-${Date.now()}`;
}

export const useAppStore = create<AppState>((set, get) => ({
  rootPath: null,
  treeData: [],
  activeTabId: null,
  tabs: [],
  settings: { ...DEFAULT_SETTINGS },
  statusMessage: 'Ready',
  sidebarWidth: 280,
  memoryUsageMB: 0,
  memoryWarning: null,

  setRootPath: (path) => set({ rootPath: path }),
  setTreeData: (data) => set({ treeData: data }),

  addTab: (pdfPath: string, title: string) => {
    const { tabs, settings } = get();

    // Check if tab with same path already exists
    const existing = tabs.find((t) => t.pdfPath === pdfPath);
    if (existing) {
      set({ activeTabId: existing.id });
      return;
    }

    // Check max tabs
    if (tabs.length >= settings.maxTabs) {
      set({
        statusMessage: `Max tabs (${settings.maxTabs}) reached. Close a tab first.`,
      });
      return;
    }

    const id = generateTabId();
    set({
      tabs: [...tabs, { id, title, pdfPath }],
      activeTabId: id,
      statusMessage: `Opened: ${title}`,
    });
  },

  closeTab: (tabId: string) => {
    const { tabs, activeTabId } = get();
    const idx = tabs.findIndex((t) => t.id === tabId);
    if (idx === -1) return;

    const newTabs = tabs.filter((t) => t.id !== tabId);
    let newActive = activeTabId;
    if (activeTabId === tabId) {
      // Activate adjacent tab
      if (newTabs.length > 0) {
        newActive = newTabs[Math.min(idx, newTabs.length - 1)].id;
      } else {
        newActive = null;
      }
    }
    set({ tabs: newTabs, activeTabId: newActive });
  },

  setActiveTab: (tabId: string | null) => {
    set({ activeTabId: tabId });
  },

  getTabIndex: (tabId: string) => {
    return get().tabs.findIndex((t) => t.id === tabId);
  },

  updateSettings: (partial) => {
    set((state) => ({
      settings: { ...state.settings, ...partial },
    }));
  },

  setMemoryUsage: (mb: number) => set({ memoryUsageMB: mb }),
  setMemoryWarning: (msg: string | null) => set({ memoryWarning: msg }),

  setStatusMessage: (msg) => set({ statusMessage: msg }),
  setSidebarWidth: (width) => set({ sidebarWidth: width }),

  reset: () =>
    set({
      rootPath: null,
      treeData: [],
      activeTabId: null,
      tabs: [],
      settings: { ...DEFAULT_SETTINGS },
      statusMessage: 'Ready',
      memoryUsageMB: 0,
      memoryWarning: null,
    }),
}));
