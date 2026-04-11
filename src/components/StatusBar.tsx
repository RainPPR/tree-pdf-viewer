import { useAppStore } from '../store/appStore';

export function StatusBar() {
  const statusMessage = useAppStore((s) => s.statusMessage);

  return (
    <div className="status-bar">
      <span>{statusMessage}</span>
    </div>
  );
}
