export interface TreeNode {
  name: string;
  path: string;
  isFolder: boolean;
  children?: TreeNode[];
}

export interface Tab {
  id: string;
  title: string;
  pdfPath: string;
}

export interface AppSettings {
  maxTabs: number;
  memoryLimitMB: number;
}

export const DEFAULT_SETTINGS: AppSettings = {
  maxTabs: 20,
  memoryLimitMB: 1024,
};

export const MIN_MEMORY_LIMIT_MB = 512;
