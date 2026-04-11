export interface TreeNode {
  name: string;
  path: string;
  isFolder: boolean;
  children?: TreeNode[];
}
