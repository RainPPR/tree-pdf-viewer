import { AlertTriangle, MemoryStick } from 'lucide-react';
import { useAppStore } from '../store/appStore';

export function StatusBar() {
  const statusMessage = useAppStore((s) => s.statusMessage);
  const memoryUsageMB = useAppStore((s) => s.memoryUsageMB);
  const memoryLimitMB = useAppStore((s) => s.settings.memoryLimitMB);
  const memoryWarning = useAppStore((s) => s.memoryWarning);

  const memoryColor = (() => {
    if (!memoryUsageMB) return 'var(--statusbar-text)';
    const ratio = memoryUsageMB / memoryLimitMB;
    if (ratio >= 1.0) return '#f38ba8';
    if (ratio >= 0.85) return '#f9e2af';
    return 'var(--statusbar-text)';
  })();

  return (
    <div className="status-bar">
      <span className="status-message">{statusMessage}</span>
      <div className="status-spacer" />
      {memoryWarning && (
        <span className="status-warning" title={memoryWarning}>
          <AlertTriangle size={12} />
          {memoryWarning}
        </span>
      )}
      {memoryUsageMB > 0 && (
        <span className="status-memory" style={{ color: memoryColor }}>
          <MemoryStick size={12} />
          {memoryUsageMB}MB / {memoryLimitMB}MB
        </span>
      )}
    </div>
  );
}
