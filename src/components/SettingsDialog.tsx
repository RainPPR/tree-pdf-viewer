import { useState, useEffect, useRef } from 'react';
import { X, AlertTriangle } from 'lucide-react';
import { useAppStore } from '../store/appStore';
import { MIN_MEMORY_LIMIT_MB } from '../types';

interface SettingsDialogProps {
  onClose: () => void;
}

export function SettingsDialog({ onClose }: SettingsDialogProps) {
  const settings = useAppStore((s) => s.settings);
  const updateSettings = useAppStore((s) => s.updateSettings);
  const [maxTabs, setMaxTabs] = useState(settings.maxTabs.toString());
  const [memoryLimitMB, setMemoryLimitMB] = useState(settings.memoryLimitMB.toString());
  const [warning, setWarning] = useState<string | null>(null);
  const overlayRef = useRef<HTMLDivElement>(null);

  const handleSave = () => {
    const newMaxTabs = parseInt(maxTabs, 10);
    const newMemoryLimit = parseInt(memoryLimitMB, 10);

    if (isNaN(newMaxTabs) || newMaxTabs < 1) {
      setWarning('Max tabs must be at least 1');
      return;
    }
    if (isNaN(newMemoryLimit) || newMemoryLimit < MIN_MEMORY_LIMIT_MB) {
      setWarning(`Memory limit must be at least ${MIN_MEMORY_LIMIT_MB} MB`);
      return;
    }

    if (newMemoryLimit < MIN_MEMORY_LIMIT_MB) {
      setWarning(`Warning: Memory limit below ${MIN_MEMORY_LIMIT_MB} MB may cause issues`);
    }

    updateSettings({
      maxTabs: newMaxTabs,
      memoryLimitMB: newMemoryLimit,
    });
    setWarning(null);
    onClose();
  };

  const handleOverlayClick = (e: React.MouseEvent) => {
    if (e.target === overlayRef.current) {
      handleSave();
    }
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') handleSave();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  return (
    <div className="modal-overlay" ref={overlayRef} onClick={handleOverlayClick}>
      <div className="modal-dialog">
        <div className="modal-header">
          <h2>Settings</h2>
          <button className="modal-close" onClick={handleSave}>
            <X size={18} strokeWidth={2} />
          </button>
        </div>

        {warning && (
          <div className="modal-warning">
            <AlertTriangle size={16} />
            <span>{warning}</span>
          </div>
        )}

        <div className="modal-body">
          <div className="setting-row">
            <label htmlFor="maxTabs">Max Tabs</label>
            <input
              id="maxTabs"
              type="number"
              min="1"
              max="50"
              value={maxTabs}
              onChange={(e) => setMaxTabs(e.target.value)}
            />
            <span className="setting-hint">Maximum number of open PDF tabs</span>
          </div>

          <div className="setting-row">
            <label htmlFor="memoryLimit">Memory Limit (MB)</label>
            <input
              id="memoryLimit"
              type="number"
              min={MIN_MEMORY_LIMIT_MB}
              max="4096"
              step="128"
              value={memoryLimitMB}
              onChange={(e) => setMemoryLimitMB(e.target.value)}
            />
            <span className="setting-hint">
              JS heap memory limit. Below {MIN_MEMORY_LIMIT_MB} MB may cause instability.
              When exceeded, oldest tabs will be auto-closed.
            </span>
          </div>
        </div>

        <div className="modal-footer">
          <button className="btn-primary" onClick={handleSave}>
            Save
          </button>
        </div>
      </div>
    </div>
  );
}
