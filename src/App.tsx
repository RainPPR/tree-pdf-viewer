import { useEffect } from 'react';
import { Toolbar } from './components/Toolbar';
import { TreeView } from './components/TreeView';
import { PdfViewer } from './components/PdfViewer';
import { StatusBar } from './components/StatusBar';
import { useAppStore } from './store/appStore';
import { useState, useCallback, useRef } from 'react';
import './App.css';

const MIN_SIDEBAR_WIDTH = 180;
const MAX_SIDEBAR_WIDTH = 600;

function App() {
  const sidebarWidth = useAppStore((s) => s.sidebarWidth);
  const setSidebarWidth = useAppStore((s) => s.setSidebarWidth);
  const [isResizing, setIsResizing] = useState(false);
  const startXRef = useRef(0);
  const startWidthRef = useRef(0);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      setIsResizing(true);
      startXRef.current = e.clientX;
      startWidthRef.current = sidebarWidth;
    },
    [sidebarWidth]
  );

  useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      const delta = e.clientX - startXRef.current;
      const newWidth = Math.min(
        MAX_SIDEBAR_WIDTH,
        Math.max(MIN_SIDEBAR_WIDTH, startWidthRef.current + delta)
      );
      setSidebarWidth(newWidth);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isResizing, setSidebarWidth]);

  return (
    <div className="app-container">
      {isResizing && (
        <div className="resize-overlay" />
      )}
      <Toolbar />
      <div className="main-content">
        <div className="sidebar" style={{ width: sidebarWidth }}>
          <TreeView />
        </div>
        <div
          className="resize-handle"
          onMouseDown={handleMouseDown}
        />
        <div className="pdf-area">
          <PdfViewer />
        </div>
      </div>
      <StatusBar />
    </div>
  );
}

export default App;
