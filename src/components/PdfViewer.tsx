import { convertFileSrc } from '@tauri-apps/api/core';
import { useEffect, useRef } from 'react';
import { useAppStore } from '../store/appStore';

function PdfIframe({ pdfPath, active }: { pdfPath: string; active: boolean }) {
  const src = convertFileSrc(pdfPath);

  return (
    <div
      className="pdf-container"
      style={{ display: active ? 'flex' : 'none' }}
    >
      <iframe
        src={src}
        title={pdfPath}
        className="pdf-iframe"
      />
    </div>
  );
}

export function PdfViewer() {
  const tabs = useAppStore((s) => s.tabs);
  const activeTabId = useAppStore((s) => s.activeTabId);
  const setMemoryUsage = useAppStore((s) => s.setMemoryUsage);
  const setMemoryWarning = useAppStore((s) => s.setMemoryWarning);
  const setStatusMessage = useAppStore((s) => s.setStatusMessage);
  const closeTab = useAppStore((s) => s.closeTab);
  const monitorRef = useRef<number | null>(null);

  // Memory monitoring
  useEffect(() => {
    const checkMemory = () => {
      const mem = (performance as any).memory;
      if (!mem || !mem.usedJSHeapSize) {
        // performance.memory not available - skip monitoring
        return;
      }

      const usedMB = mem.usedJSHeapSize / (1024 * 1024);
      setMemoryUsage(Math.round(usedMB));

      const limitMB = useAppStore.getState().settings.memoryLimitMB;
      const ratio = usedMB / limitMB;

      if (ratio >= 1.0) {
        // Memory limit exceeded - start closing oldest inactive tabs
        const state = useAppStore.getState();
        const currentTabId = state.activeTabId;
        const inactiveTabs = state.tabs.filter((t) => t.id !== currentTabId);

        if (inactiveTabs.length > 0) {
          // Close the oldest inactive tab
          const oldest = inactiveTabs[0];
          closeTab(oldest.id);
          setMemoryWarning(
            `Memory limit reached (${Math.round(usedMB)}MB / ${limitMB}MB). Closed "${oldest.title}".`
          );
          setStatusMessage(
            `Auto-closed tab: ${oldest.title} (memory limit exceeded)`
          );
        } else {
          // No more tabs to close - fatal error
          setMemoryWarning(
            `CRITICAL: Memory limit exceeded (${Math.round(usedMB)}MB / ${limitMB}MB) and no tabs to close. Application may become unstable.`
          );
          setStatusMessage(
            `FATAL: Memory limit exceeded and all tabs closed. Please restart the application.`
          );
          // Stop monitoring - we've done all we can
          if (monitorRef.current) {
            clearInterval(monitorRef.current);
            monitorRef.current = null;
          }
        }
      } else if (ratio >= 0.85) {
        setMemoryWarning(
          `Memory usage high: ${Math.round(usedMB)}MB / ${limitMB}MB (${Math.round(ratio * 100)}%)`
        );
      } else {
        setMemoryWarning(null);
      }
    };

    // Check every 5 seconds
    monitorRef.current = window.setInterval(checkMemory, 5000);
    // Also check immediately
    checkMemory();

    return () => {
      if (monitorRef.current) {
        clearInterval(monitorRef.current);
        monitorRef.current = null;
      }
    };
  }, []); // Run once on mount, uses getState() for current data

  if (tabs.length === 0) {
    return (
      <div className="pdf-viewer pdf-empty">
        <div className="pdf-placeholder">
          <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
          <p>Select a PDF file from the tree to view</p>
        </div>
      </div>
    );
  }

  return (
    <div className="pdf-viewer">
      {tabs.map((tab) => (
        <PdfIframe
          key={tab.id}
          pdfPath={tab.pdfPath}
          active={tab.id === activeTabId}
        />
      ))}
    </div>
  );
}
