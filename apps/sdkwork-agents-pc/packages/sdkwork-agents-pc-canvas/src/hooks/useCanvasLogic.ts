import React, { useState, useRef, useEffect, useCallback } from 'react';
import { CanvasNode, Connection, CanvasTool, PanPosition, CanvasGroup } from '../types';
import { CanvasService } from '../services/CanvasService';
import { exportCanvasToPDF } from '../utils/pdfExport';
import { getAdaptedHeight, getNumericAspectRatio } from '../utils/ratioHelper';


export function useCanvasLogic() {
  const containerRef = useRef<HTMLDivElement>(null);
  
  // Canvas Nodes State
  const [nodes, setNodes] = useState<CanvasNode[]>([]);

  // Canvas Groups State
  const [groups, setGroups] = useState<CanvasGroup[]>([]);

  // Connections State
  const [connections, setConnections] = useState<Connection[]>([]);

  const [isLoaded, setIsLoaded] = useState(false);

  // Global Toast Notification State
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' | 'info' | 'loading' } | null>(null);

  // Context Menu State
  const [contextMenu, setContextMenu] = useState<{ x: number, y: number, target: 'canvas' | 'node' | 'group' | 'connection' } | null>(null);

  useEffect(() => {
    CanvasService.getInitialWorkflow().then(data => {
      setNodes(data.nodes);
      setGroups(data.groups);
      setConnections(data.connections);
      setIsLoaded(true);
    });
  }, []);

  useEffect(() => {
    if (isLoaded) {
      CanvasService.saveWorkflow({ nodes, groups, connections });
    }
  }, [nodes, groups, connections, isLoaded]);

  // Viewport transforms
  const [pan, setPan] = useState<PanPosition>({ x: 50, y: 50 });
  const [zoom, setZoom] = useState<number>(0.90);
  const [isAnimatingView, setIsAnimatingView] = useState(false);
  const animationTimerRef = useRef<NodeJS.Timeout | null>(null);

  // Viewport Size tracking and Minimap show state
  const [viewportSize, setViewportSize] = useState({ width: 800, height: 600 });
  const [showMinimap, setShowMinimap] = useState<boolean>(true);

  useEffect(() => {
    if (!containerRef.current) return;

    setViewportSize({
      width: containerRef.current.clientWidth || 800,
      height: containerRef.current.clientHeight || 600
    });

    const observer = new ResizeObserver((entries) => {
      for (let entry of entries) {
        if (containerRef.current) {
          setViewportSize({
            width: containerRef.current.clientWidth,
            height: containerRef.current.clientHeight
          });
        }
      }
    });
    observer.observe(containerRef.current);

    return () => observer.disconnect();
  }, [isLoaded]);

  const setSmoothView = (newPan: {x: number, y: number}, newZoom: number) => {
    setIsAnimatingView(true);
    setPan(newPan);
    setZoom(newZoom);
    if (animationTimerRef.current) clearTimeout(animationTimerRef.current);
    animationTimerRef.current = setTimeout(() => {
      setIsAnimatingView(false);
    }, 400); // match css transition duration
  };

  const [clipboard, setClipboard] = useState<{ nodes: CanvasNode[] }>({ nodes: [] });
  const [activeTool, setActiveTool] = useState<CanvasTool>('select');
  
  // Selection States
  const [selectedNodeIds, setSelectedNodeIds] = useState<string[]>([]);
  const [selectedConnectionId, setSelectedConnectionId] = useState<string | null>(null);
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(null);

  // Dragging States
  const [isPanning, setIsPanning] = useState(false);
  const [panStart, setPanStart] = useState({ x: 0, y: 0 });
  
  const [draggingNodeId, setDraggingNodeId] = useState<string | null>(null);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });
  const [draggingConnectionId, setDraggingConnectionId] = useState<string | null>(null);

  // Group Dragging & Resizing States
  const [draggingGroupId, setDraggingGroupId] = useState<string | null>(null);
  const [dragGroupOffset, setDragGroupOffset] = useState({ x: 0, y: 0 });

  const [resizingGroupId, setResizingGroupId] = useState<string | null>(null);
  const [resizeStartSize, setResizeStartSize] = useState({ width: 0, height: 0 });
  const [resizeStartMouse, setResizeStartMouse] = useState({ x: 0, y: 0 });

  // Node Resizing States
  const [resizingNodeId, setResizingNodeId] = useState<string | null>(null);
  const [resizeNodeDirection, setResizeNodeDirection] = useState<'w' | 'h' | 'both'>('both');
  const [resizeNodeStartSize, setResizeNodeStartSize] = useState({ width: 0, height: 0 });
  const [resizeNodeStartMouse, setResizeNodeStartMouse] = useState({ x: 0, y: 0 });
  const [snapToGrid, setSnapToGrid] = useState<boolean>(true);
  const [showGrid, setShowGrid] = useState<boolean>(true);

  // Lasso Selection State
  const [selectionBox, setSelectionBox] = useState<{
    startX: number;
    startY: number;
    currentX: number;
    currentY: number;
  } | null>(null);

  // Connection Dragging & Snapping States
  const [activePortDrag, setActivePortDrag] = useState<{
    nodeId: string;
    type: 'input' | 'output';
    startX: number;
    startY: number;
  } | null>(null);
  const [portDragCurrentPos, setPortDragCurrentPos] = useState({ x: 0, y: 0 });
  const [snappingTargetNodeId, setSnappingTargetNodeId] = useState<string | null>(null);
  const [isStickyConnection, setIsStickyConnection] = useState<boolean>(false);
  const portDragStartMouseRef = useRef({ x: 0, y: 0 });

  // Help guides
  const [showHelp, setShowHelp] = useState<boolean>(() => {
    try {
      const saved = localStorage.getItem('canvas_showHelp');
      return saved !== null ? JSON.parse(saved) : true;
    } catch (e) {
      return true;
    }
  });

  useEffect(() => {
    localStorage.setItem('canvas_showHelp', JSON.stringify(showHelp));
  }, [showHelp]);
  const [showSnapshots, setShowSnapshots] = useState<boolean>(false);
  const [showTemplates, setShowTemplates] = useState<boolean>(false);
  const [spacePressed, setSpacePressed] = useState(false);

  // --- EXTREME POLISH: ADVANCED DRAG & PERSISTENCE STATES ---
  interface AlignGuide {
    id: string;
    type: 'h' | 'v'; // h = horizontal line, v = vertical line
    coord: number;   // target axis coordinate
    start: number;   // bounding box line start
    end: number;     // bounding box line end
    label?: string;  // descriptive label
  }
  const [alignGuides, setAlignGuides] = useState<AlignGuide[]>([]);

  // Position & structural history states for seamless undo / redo
  type HistoryState = {
    nodes: CanvasNode[];
    groups: CanvasGroup[];
    connections: Connection[];
  };
  const [history, setHistory] = useState<HistoryState[]>([]);
  const [redoStack, setRedoStack] = useState<HistoryState[]>([]);
  const dragInitialStateRef = useRef<HistoryState | null>(null);
  const nodeDragMovedRef = useRef<boolean>(false);
  const lastShiftDeselectionCandidateRef = useRef<string | null>(null);
  const lastDeselectionCandidateRef = useRef<string | null>(null);
  const nextSelectedNodeIdsRef = useRef<string[] | null>(null);

  // Refs for tracking synchronous drag metrics across state boundaries
  const dragNodesOffsetsRef = useRef<Record<string, { x: number; y: number }>>({});
  const dragGroupNodesOffsetsRef = useRef<Record<string, { x: number; y: number }>>({});
  const panRef = useRef(pan);
  const zoomRef = useRef(zoom);
  const nodesRef = useRef<CanvasNode[]>(nodes);
  const groupsRef = useRef<CanvasGroup[]>(groups);
  const connectionsRef = useRef<Connection[]>(connections);
  const lastTextSaveRef = useRef<Record<string, number>>({});

  // Sync references to eliminate stale closures
  useEffect(() => {
    panRef.current = pan;
  }, [pan]);

  useEffect(() => {
    zoomRef.current = zoom;
  }, [zoom]);

  useEffect(() => {
    nodesRef.current = nodes;
  }, [nodes]);

  useEffect(() => {
    groupsRef.current = groups;
  }, [groups]);

  useEffect(() => {
    connectionsRef.current = connections;
  }, [connections]);

  // High-performance edge auto-panning loops
  const edgePanIntervalRef = useRef<number | null>(null);
  const mouseScreenPosRef = useRef({ x: 0, y: 0 });

  const saveHistory = useCallback(() => {
    setHistory(prev => {
      const updated = [...prev, {
        nodes: nodesRef.current,
        groups: groupsRef.current,
        connections: connectionsRef.current
      }];
      if (updated.length > 30) updated.shift();
      return updated;
    });
    setRedoStack([]);
  }, []);

  const handleUndo = useCallback(() => {
    setHistory(prevHistory => {
      if (prevHistory.length === 0) return prevHistory;
      const prev = prevHistory[prevHistory.length - 1];
      setRedoStack(r => [...r, {
        nodes: nodesRef.current,
        groups: groupsRef.current,
        connections: connectionsRef.current
      }]);
      setNodes(prev.nodes);
      setGroups(prev.groups);
      setConnections(prev.connections);
      return prevHistory.slice(0, -1);
    });
  }, []);

  const handleRedo = useCallback(() => {
    setRedoStack(prevRedo => {
      if (prevRedo.length === 0) return prevRedo;
      const next = prevRedo[prevRedo.length - 1];
      setHistory(h => [...h, {
        nodes: nodesRef.current,
        groups: groupsRef.current,
        connections: connectionsRef.current
      }]);
      setNodes(next.nodes);
      setGroups(next.groups);
      setConnections(next.connections);
      return prevRedo.slice(0, -1);
    });
  }, []);

  // Hotkey handles
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const activeEl = document.activeElement?.tagName;
      if (activeEl === 'INPUT' || activeEl === 'TEXTAREA') return;

      if (e.code === 'Space') {
        e.preventDefault();
        setSpacePressed(true);
      }
      if (e.code === 'KeyV' && !(e.ctrlKey || e.metaKey)) {
        setActiveTool('select');
      }
      if (e.code === 'KeyH') {
        setActiveTool('hand');
      }
      if (e.code === 'Escape') {
        setSelectedNodeIds([]);
        setSelectedConnectionId(null);
        setSelectedGroupId(null);
        setActivePortDrag(null);
        setSelectionBox(null);
        setIsStickyConnection(false);
      }
      if (e.code === 'Delete' || e.code === 'Backspace') {
        // Handle deletion of selected elements
        if (selectedNodeIds.length > 0) {
          handleBatchDelete();
        } else if (selectedConnectionId) {
          saveHistory();
          setConnections(prev => prev.filter(c => c.id !== selectedConnectionId));
          setSelectedConnectionId(null);
        } else if (selectedGroupId) {
          handleDisbandGroup(selectedGroupId);
          setSelectedGroupId(null);
        }
      }

      // Copy
      if ((e.ctrlKey || e.metaKey) && e.code === 'KeyC') {
        if (selectedNodeIds.length > 0) {
          const nodesToCopy = nodes.filter(n => selectedNodeIds.includes(n.id));
          setClipboard({ nodes: nodesToCopy });
        }
      }
      
      // Duplicate (Ctrl+D)
      if ((e.ctrlKey || e.metaKey) && e.code === 'KeyD') {
        e.preventDefault();
        if (selectedNodeIds.length > 0) {
          saveHistory();
          const nodesToDuplicate = nodes.filter(n => selectedNodeIds.includes(n.id));
          const newNodes = nodesToDuplicate.map(n => ({
            ...n,
            id: `node-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`,
            x: n.x + 40,
            y: n.y + 40,
            groupId: undefined
          }));
          setNodes(prev => [...prev, ...newNodes]);
          setSelectedNodeIds(newNodes.map(n => n.id));
        }
      }

      // Group (Ctrl+G) / Ungroup (Ctrl+Shift+G)
      if ((e.ctrlKey || e.metaKey) && e.code === 'KeyG') {
        e.preventDefault();
        if (e.shiftKey) {
          // Ungroup (Disband)
          if (selectedGroupId) {
            handleDisbandGroup(selectedGroupId);
            setSelectedGroupId(null);
          } else if (selectedNodeIds.length === 1) {
            const nodeId = selectedNodeIds[0];
            const node = nodes.find(n => n.id === nodeId);
            if (node && node.groupId) {
              const oldGroupId = node.groupId;
              saveHistory();
              setNodes(prev => prev.map(n => n.id === nodeId ? { ...n, groupId: undefined } : n));
              setGroups(prev => prev.map(g => g.id === oldGroupId ? { ...g, nodeIds: g.nodeIds.filter(id => id !== nodeId) } : g).filter(g => g.nodeIds.length > 0 || g.id === 'group-initial'));
              showToastMessage('已将卡片移出分组', 'success');
            }
          }
        } else {
          // Group
          if (selectedNodeIds.length > 0) {
            handleCreateGroupFromSelection();
          }
        }
      }

      // Paste
      if ((e.ctrlKey || e.metaKey) && e.code === 'KeyV') {
        if (clipboard.nodes.length > 0) {
          saveHistory();
          const newNodes = clipboard.nodes.map(n => ({
            ...n,
            id: `node-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`,
            x: n.x + 50,
            y: n.y + 50,
            groupId: undefined // remove from group on paste
          }));
          setNodes(prev => [...prev, ...newNodes]);
          setSelectedNodeIds(newNodes.map(n => n.id));
        }
      }

      // Ctrl + Z for undo, Ctrl + Y for redo
      if ((e.ctrlKey || e.metaKey) && e.code === 'KeyZ') {
        e.preventDefault();
        handleUndo();
      }
      if ((e.ctrlKey || e.metaKey) && (e.code === 'KeyY' || (e.shiftKey && e.code === 'KeyZ'))) {
        e.preventDefault();
        handleRedo();
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.code === 'Space') {
        setSpacePressed(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [selectedNodeIds, selectedConnectionId, selectedGroupId, nodes, groups, connections, history, redoStack, clipboard]);

  // Context Menu Action Handler
  const handleContextMenuAction = (action: string) => {
    switch(action) {
      case 'add-text':
        handleAddNode('text');
        break;
      case 'add-image':
        handleAddNode('image-gen');
        break;
      case 'add-video':
        handleAddNode('video-gen');
        break;
      case 'add-sticky':
        handleAddNode('sticky');
        break;
      case 'paste':
        if (clipboard.nodes.length > 0) {
          const newNodes = clipboard.nodes.map(n => ({
            ...n,
            id: `node-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`,
            x: n.x + 50,
            y: n.y + 50,
            groupId: undefined
          }));
          setNodes(prev => [...prev, ...newNodes]);
          setSelectedNodeIds(newNodes.map(n => n.id));
        }
        break;
      case 'select-all':
        setSelectedNodeIds(nodes.map(n => n.id));
        break;
      case 'copy':
        if (selectedNodeIds.length > 0) {
          const nodesToCopy = nodes.filter(n => selectedNodeIds.includes(n.id));
          setClipboard({ nodes: nodesToCopy });
        }
        break;
      case 'duplicate':
        if (selectedNodeIds.length > 0) {
          saveHistory();
          const nodesToDuplicate = nodes.filter(n => selectedNodeIds.includes(n.id));
          const newNodes = nodesToDuplicate.map(n => ({
            ...n,
            id: `node-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`,
            x: n.x + 40,
            y: n.y + 40,
            groupId: undefined
          }));
          setNodes(prev => [...prev, ...newNodes]);
          setSelectedNodeIds(newNodes.map(n => n.id));
        }
        break;
      case 'create-group':
        handleCreateGroupFromSelection();
        break;
      case 'remove-from-group':
        if (selectedNodeIds.length === 1) {
          saveHistory();
          const nodeId = selectedNodeIds[0];
          const node = nodes.find(n => n.id === nodeId);
          if (node && node.groupId) {
            const oldGroupId = node.groupId;
            setNodes(prev => prev.map(n => n.id === nodeId ? { ...n, groupId: undefined } : n));
            setGroups(prev => prev.map(g => g.id === oldGroupId ? { ...g, nodeIds: g.nodeIds.filter(id => id !== nodeId) } : g).filter(g => g.nodeIds.length > 0 || g.id === 'group-initial'));
            showToastMessage('已将卡片移出当前分组', 'success');
          }
        }
        break;
      case 'disband-group':
        if (selectedGroupId) {
          handleDisbandGroup(selectedGroupId);
          setSelectedGroupId(null);
        }
        break;
      case 'delete':
        if (selectedNodeIds.length > 0) {
          handleBatchDelete();
        } else if (selectedGroupId) {
          handleDisbandGroup(selectedGroupId);
          setSelectedGroupId(null);
        }
        break;
      case 'delete-conn':
        if (selectedConnectionId) {
          saveHistory();
          setConnections(prev => prev.filter(c => c.id !== selectedConnectionId));
          setSelectedConnectionId(null);
        }
        break;
      case 'zoom-to-fit':
        handleResetView();
        break;
      case 'export':
        handleExport();
        break;
      case 'clear-workspace':
        handleClearCanvas();
        break;
    }
    setContextMenu(null);
  };

  const zoomToPoint = (targetZoom: number, mouseX: number, mouseY: number) => {
    const canvasX = (mouseX - pan.x) / zoom;
    const canvasY = (mouseY - pan.y) / zoom;
    
    const newPanX = mouseX - canvasX * targetZoom;
    const newPanY = mouseY - canvasY * targetZoom;
    
    setSmoothView({ x: newPanX, y: newPanY }, targetZoom);
  };

  const handleZoomIn = () => {
    if (!containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    const targetZoom = Math.min(2.0, Number((zoom + 0.1).toFixed(2)));
    zoomToPoint(targetZoom, rect.width / 2, rect.height / 2);
  };

  const handleZoomOut = () => {
    if (!containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    const targetZoom = Math.max(0.15, Number((zoom - 0.1).toFixed(2)));
    zoomToPoint(targetZoom, rect.width / 2, rect.height / 2);
  };

  const handleResetZoom = () => {
    if (!containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    zoomToPoint(1.0, rect.width / 2, rect.height / 2);
  };

  // Wheel Handler (Zoom & Pan)
  const handleWheel = (e: React.WheelEvent) => {
    // We cannot preventDefault on React passive wheel events safely without a ref in some cases,
    // but we can try. If it's a pinch, e.ctrlKey is true.
    if (!containerRef.current) return;
    
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      const zoomIntensity = 0.01;
      const rect = containerRef.current.getBoundingClientRect();
      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      const canvasX = (mouseX - pan.x) / zoom;
      const canvasY = (mouseY - pan.y) / zoom;

      // When pinching on trackpad, e.deltaY corresponds to zoom scale delta.
      const delta = -e.deltaY;
      const zoomFactor = delta > 0 ? (1 + zoomIntensity * Math.abs(delta)) : (1 - zoomIntensity * Math.abs(delta));
      const newZoom = Math.max(0.15, Math.min(2.0, zoom * zoomFactor));

      const newPanX = mouseX - canvasX * newZoom;
      const newPanY = mouseY - canvasY * newZoom;

      setZoom(Number(newZoom.toFixed(3)));
      setPan({ x: newPanX, y: newPanY });
    } else {
      // Pan
      setPan(prev => ({
        x: prev.x - e.deltaX,
        y: prev.y - e.deltaY
      }));
    }
  };

  // Mouse Down Event on Canvas
  const handleMouseDown = (e: React.MouseEvent) => {
    if (!containerRef.current) return;

    // Middle click or hand tool triggers drag pan
    if (e.button === 1 || activeTool === 'hand' || spacePressed) {
      setIsPanning(true);
      setPanStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
      e.preventDefault();
      return;
    }

    if (e.button !== 0) return; // Left click only

    const target = e.target as HTMLElement;
    
    // If connection line is active and we click the background, cancel it
    if (activePortDrag && (target.classList.contains('canvas-background') || target.classList.contains('grid-svg'))) {
      setActivePortDrag(null);
      setSnappingTargetNodeId(null);
      setIsStickyConnection(false);
      return;
    }

    // Clicked background -> deselect & start selection box
    if (target.classList.contains('canvas-background') || target.classList.contains('grid-svg')) {
      if (!e.shiftKey) {
        setSelectedNodeIds([]);
        setSelectedConnectionId(null);
        setSelectedGroupId(null);
      }
      
      // Starts drawing selection box in select mode
      const rect = containerRef.current.getBoundingClientRect();
      const worldX = (e.clientX - rect.left - pan.x) / zoom;
      const worldY = (e.clientY - rect.top - pan.y) / zoom;
      setSelectionBox({
        startX: worldX,
        startY: worldY,
        currentX: worldX,
        currentY: worldY
      });
    }
  };

  // --- EXTREME POLISH: INTEGRATED HELPERS FOR CANVAS DRAGGING & PANNING ---
  const calculateSnapping = (
    dragId: string,
    targetX: number,
    targetY: number,
    allNodes: CanvasNode[]
  ) => {
    const dragNode = allNodes.find(n => n.id === dragId);
    if (!dragNode) return { x: targetX, y: targetY, guides: [] };

    const w = dragNode.width || 320;
    const h = dragNode.height || 300;

    const snapThreshold = 8; // Snap within 8 world pixels
    let snappedX = targetX;
    let snappedY = targetY;

    if (snapToGrid) {
      snappedX = Math.round(targetX / 16) * 16;
      snappedY = Math.round(targetY / 16) * 16;
    }

    const guides: AlignGuide[] = [];

    // Only align with reference nodes that are NOT being dragged
    const selectedSet = new Set(Object.keys(dragNodesOffsetsRef.current));
    const referenceNodes = allNodes.filter(n => !selectedSet.has(n.id));

    let xSnapped = false;
    let ySnapped = false;

    // 1. Horizontal Snapping (Vertical guide lines)
    for (const refNode of referenceNodes) {
      if (xSnapped) break;
      const refW = refNode.width || 320;
      const refH = refNode.height || 300;

      const refLeft = refNode.x;
      const refRight = refNode.x + refW;
      const refCenterX = refNode.x + refW / 2;

      const dragLeft = targetX;
      const dragRight = targetX + w;
      const dragCenterX = targetX + w / 2;

      // Check Left - Left alignment
      if (Math.abs(dragLeft - refLeft) < snapThreshold) {
        snappedX = refLeft;
        xSnapped = true;
        guides.push({
          id: `v-left-${refNode.id}`,
          type: 'v',
          coord: refLeft,
          start: Math.min(targetY, refNode.y),
          end: Math.max(targetY + h, refNode.y + refH),
          label: '左侧对齐'
        });
      }
      // Check Right - Right alignment
      else if (Math.abs(dragRight - refRight) < snapThreshold) {
        snappedX = refRight - w;
        xSnapped = true;
        guides.push({
          id: `v-right-${refNode.id}`,
          type: 'v',
          coord: refRight,
          start: Math.min(targetY, refNode.y),
          end: Math.max(targetY + h, refNode.y + refH),
          label: '右侧对齐'
        });
      }
      // Check Center - Center alignment
      else if (Math.abs(dragCenterX - refCenterX) < snapThreshold) {
        snappedX = refCenterX - w / 2;
        xSnapped = true;
        guides.push({
          id: `v-center-${refNode.id}`,
          type: 'v',
          coord: refCenterX,
          start: Math.min(targetY, refNode.y),
          end: Math.max(targetY + h, refNode.y + refH),
          label: '水平对齐'
        });
      }
    }

    // 2. Vertical Snapping (Horizontal guide lines)
    for (const refNode of referenceNodes) {
      if (ySnapped) break;
      const refW = refNode.width || 320;
      const refH = refNode.height || 300;

      const refTop = refNode.y;
      const refBottom = refNode.y + refH;
      const refCenterY = refNode.y + refH / 2;

      const dragTop = targetY;
      const dragBottom = targetY + h;
      const dragCenterY = targetY + h / 2;

      // Check Top - Top alignment
      if (Math.abs(dragTop - refTop) < snapThreshold) {
        snappedY = refTop;
        ySnapped = true;
        guides.push({
          id: `h-top-${refNode.id}`,
          type: 'h',
          coord: refTop,
          start: Math.min(targetX, refNode.x),
          end: Math.max(targetX + w, refNode.x + refW),
          label: '顶部对齐'
        });
      }
      // Check Bottom - Bottom alignment
      else if (Math.abs(dragBottom - refBottom) < snapThreshold) {
        snappedY = refBottom - h;
        ySnapped = true;
        guides.push({
          id: `h-bottom-${refNode.id}`,
          type: 'h',
          coord: refBottom,
          start: Math.min(targetX, refNode.x),
          end: Math.max(targetX + w, refNode.x + refW),
          label: '底部对齐'
        });
      }
      // Check Center - Center alignment
      else if (Math.abs(dragCenterY - refCenterY) < snapThreshold) {
        snappedY = refCenterY - h / 2;
        ySnapped = true;
        guides.push({
          id: `h-center-${refNode.id}`,
          type: 'h',
          coord: refCenterY,
          start: Math.min(targetX, refNode.x),
          end: Math.max(targetX + w, refNode.x + refW),
          label: '垂直对齐'
        });
      }
    }

    return { x: snappedX, y: snappedY, guides };
  };

  const calculateResizeSnapping = (
    resizeId: string,
    targetWidth: number,
    targetHeight: number,
    allNodes: CanvasNode[]
  ) => {
    const node = allNodes.find(n => n.id === resizeId);
    if (!node) return { width: targetWidth, height: targetHeight, guides: [] };

    const snapThreshold = 8;
    let snappedWidth = targetWidth;
    let snappedHeight = targetHeight;
    const guides: AlignGuide[] = [];

    const referenceNodes = allNodes.filter(n => n.id !== resizeId);

    let xSnapped = false;
    let ySnapped = false;

    // Right Edge Alignment (vertical lines)
    for (const refNode of referenceNodes) {
      if (xSnapped) break;
      const refW = refNode.width || 320;
      const refH = refNode.height || 300;

      const refLeft = refNode.x;
      const refRight = refNode.x + refW;

      const dragRight = node.x + targetWidth;

      // Align right edge with reference's left edge
      if (Math.abs(dragRight - refLeft) < snapThreshold) {
        snappedWidth = refLeft - node.x;
        xSnapped = true;
        guides.push({
          id: `resize-v-left-${refNode.id}`,
          type: 'v',
          coord: refLeft,
          start: Math.min(node.y, refNode.y),
          end: Math.max(node.y + targetHeight, refNode.y + refH),
          label: '右边缘与左侧对齐'
        });
      }
      // Align right edge with reference's right edge
      else if (Math.abs(dragRight - refRight) < snapThreshold) {
        snappedWidth = refRight - node.x;
        xSnapped = true;
        guides.push({
          id: `resize-v-right-${refNode.id}`,
          type: 'v',
          coord: refRight,
          start: Math.min(node.y, refNode.y),
          end: Math.max(node.y + targetHeight, refNode.y + refH),
          label: '宽度对齐'
        });
      }
    }

    // Bottom Edge Alignment (horizontal lines)
    for (const refNode of referenceNodes) {
      if (ySnapped) break;
      const refW = refNode.width || 320;
      const refH = refNode.height || 300;

      const refTop = refNode.y;
      const refBottom = refNode.y + refH;

      const dragBottom = node.y + targetHeight;

      // Align bottom edge with reference's top edge
      if (Math.abs(dragBottom - refTop) < snapThreshold) {
        snappedHeight = refTop - node.y;
        ySnapped = true;
        guides.push({
          id: `resize-h-top-${refNode.id}`,
          type: 'h',
          coord: refTop,
          start: Math.min(node.x, refNode.x),
          end: Math.max(node.x + targetWidth, refNode.x + refW),
          label: '下边缘与顶部对齐'
        });
      }
      // Align bottom edge with reference's bottom edge
      else if (Math.abs(dragBottom - refBottom) < snapThreshold) {
        snappedHeight = refBottom - node.y;
        ySnapped = true;
        guides.push({
          id: `resize-h-bottom-${refNode.id}`,
          type: 'h',
          coord: refBottom,
          start: Math.min(node.x, refNode.x),
          end: Math.max(node.x + targetWidth, refNode.x + refW),
          label: '高度对齐'
        });
      }
    }

    return { width: snappedWidth, height: snappedHeight, guides };
  };

  const updateDragNodePositions = (clientX: number, clientY: number) => {
    if (!containerRef.current || !draggingNodeId) return;
    const rect = containerRef.current.getBoundingClientRect();

    const currentPan = panRef.current;
    const currentZoom = zoomRef.current;

    const worldX = (clientX - rect.left - currentPan.x) / currentZoom;
    const worldY = (clientY - rect.top - currentPan.y) / currentZoom;

    setNodes(prev => {
      const draggedNode = prev.find(n => n.id === draggingNodeId);
      if (!draggedNode) return prev;

      const offset = dragNodesOffsetsRef.current[draggingNodeId];
      if (!offset) return prev;

      let targetX = worldX - offset.x;
      let targetY = worldY - offset.y;

      // Apply smart alignment snapping and guides on primary dragged node
      const snapResult = calculateSnapping(draggingNodeId, targetX, targetY, prev);
      targetX = snapResult.x;
      targetY = snapResult.y;

      setAlignGuides(snapResult.guides);

      // Translate offsets relative to primary node delta shifts
      const dx = targetX - draggedNode.x;
      const dy = targetY - draggedNode.y;

      if (dx === 0 && dy === 0) return prev;

      nodeDragMovedRef.current = true;

      const updatedNodes = prev.map(node => {
        if (node.id === draggingNodeId) {
          return {
            ...node,
            x: Math.round(targetX),
            y: Math.round(targetY)
          };
        }

        // Multi-card drag: translation of other active selected nodes
        const hasOffset = dragNodesOffsetsRef.current[node.id];
        if (hasOffset) {
          return {
            ...node,
            x: Math.round(node.x + dx),
            y: Math.round(node.y + dy)
          };
        }
        return node;
      });

      // Dynamically adjust group boundaries in real-time when dragging nodes inside a group
      const affectedGroupIds = new Set<string>();
      updatedNodes.forEach(node => {
        const isDragged = node.id === draggingNodeId || !!dragNodesOffsetsRef.current[node.id];
        if (isDragged && node.groupId) {
          affectedGroupIds.add(node.groupId);
        }
      });

      if (affectedGroupIds.size > 0) {
        setGroups(currentGroups => {
          return currentGroups.map(group => {
            if (affectedGroupIds.has(group.id)) {
              const groupNodes = updatedNodes.filter(n => group.nodeIds.includes(n.id));
              if (groupNodes.length === 0) return group;

              let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
              groupNodes.forEach(n => {
                minX = Math.min(minX, n.x);
                minY = Math.min(minY, n.y);
                maxX = Math.max(maxX, n.x + (n.width || 320));
                maxY = Math.max(maxY, n.y + (n.height || 300));
              });

              const pad = 45;
              return {
                ...group,
                x: minX - pad,
                y: minY - pad - 40,
                width: maxX - minX + pad * 2,
                height: maxY - minY + pad * 2 + 40
              };
            }
            return group;
          });
        });
      }

      return updatedNodes;
    });
  };

  const startEdgePanLoop = () => {
    if (edgePanIntervalRef.current) return;

    edgePanIntervalRef.current = window.setInterval(() => {
      if (!containerRef.current) return;
      
      // Auto-stop loop when no dragging operations are active
      if (!draggingNodeId && !activePortDrag) {
        stopEdgePanLoop();
        return;
      }

      const rect = containerRef.current.getBoundingClientRect();
      const mouseX = mouseScreenPosRef.current.x - rect.left;
      const mouseY = mouseScreenPosRef.current.y - rect.top;

      const threshold = 55; // Edge threshold margin in pixels
      let panDX = 0;
      let panDY = 0;

      // Horizontal boundaries check
      if (mouseX < threshold && mouseX > 0) {
        const intensity = (threshold - mouseX) / threshold;
        panDX = Math.round(intensity * 12);
      } else if (mouseX > rect.width - threshold && mouseX < rect.width) {
        const intensity = (mouseX - (rect.width - threshold)) / threshold;
        panDX = -Math.round(intensity * 12);
      }

      // Vertical boundaries check
      if (mouseY < threshold && mouseY > 0) {
        const intensity = (threshold - mouseY) / threshold;
        panDY = Math.round(intensity * 12);
      } else if (mouseY > rect.height - threshold && mouseY < rect.height) {
        const intensity = (mouseY - (rect.height - threshold)) / threshold;
        panDY = -Math.round(intensity * 12);
      }

      if (panDX !== 0 || panDY !== 0) {
        // Translate viewport coordinates
        setPan(prev => ({ x: prev.x + panDX, y: prev.y + panDY }));

        // Propagate translations to dragged elements
        if (draggingNodeId) {
          updateDragNodePositions(mouseScreenPosRef.current.x, mouseScreenPosRef.current.y);
        } else if (activePortDrag) {
          // Sync live wire connection point
          const currPan = { x: panRef.current.x + panDX, y: panRef.current.y + panDY };
          const worldX = (mouseScreenPosRef.current.x - rect.left - currPan.x) / zoomRef.current;
          const worldY = (mouseScreenPosRef.current.y - rect.top - currPan.y) / zoomRef.current;
          
          setPortDragCurrentPos({ x: worldX, y: worldY });
        }
      }
    }, 16);
  };

  const stopEdgePanLoop = () => {
    if (edgePanIntervalRef.current) {
      clearInterval(edgePanIntervalRef.current);
      edgePanIntervalRef.current = null;
    }
  };

  // Cleanup auto panner loop on unmount
  useEffect(() => {
    return () => {
      stopEdgePanLoop();
    };
  }, []);

  // Mouse Move Event on Canvas
  const handleMouseMove = (e: React.MouseEvent) => {
    // Record screen positions for edge panner usage
    mouseScreenPosRef.current = { x: e.clientX, y: e.clientY };

    if (isPanning) {
      setPan({
        x: e.clientX - panStart.x,
        y: e.clientY - panStart.y
      });
      return;
    }

    // Dragging Connection Control Point
    if (draggingConnectionId) {
      if (!containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const worldX = (e.clientX - rect.left - pan.x) / zoom;
      const worldY = (e.clientY - rect.top - pan.y) / zoom;

      setConnections(prev => prev.map(conn => 
        conn.id === draggingConnectionId 
          ? { ...conn, controlPoint: { x: Math.round(worldX), y: Math.round(worldY) } }
          : conn
      ));
      return;
    }

    // Dragging Group Frame -> coupling move group + nested nodes together
    if (draggingGroupId) {
      if (!containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const worldX = (e.clientX - rect.left - pan.x) / zoom - dragGroupOffset.x;
      const worldY = (e.clientY - rect.top - pan.y) / zoom - dragGroupOffset.y;

      // Update group position directly and nested nodes using absolute offsets in parallel
      setGroups(prevGroups => prevGroups.map(g => g.id === draggingGroupId ? { ...g, x: worldX, y: worldY } : g));
      setNodes(prevNodes => prevNodes.map(n => {
        const offset = dragGroupNodesOffsetsRef.current[n.id];
        if (offset) {
          return {
            ...n,
            x: Math.round(worldX + offset.x),
            y: Math.round(worldY + offset.y)
          };
        }
        return n;
      }));
      return;
    }

    // Resizing Group Frame
    if (resizingGroupId) {
      if (!containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const worldX = (e.clientX - rect.left - pan.x) / zoom;
      const worldY = (e.clientY - rect.top - pan.y) / zoom;

      const dx = worldX - resizeStartMouse.x;
      const dy = worldY - resizeStartMouse.y;

      setGroups(prev => prev.map(g => {
        if (g.id === resizingGroupId) {
          return {
            ...g,
            width: Math.max(240, resizeStartSize.width + dx),
            height: Math.max(180, resizeStartSize.height + dy)
          };
        }
        return g;
      }));
      return;
    }

    // Resizing Node Card
    if (resizingNodeId) {
      if (!containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const worldX = (e.clientX - rect.left - pan.x) / zoom;
      const worldY = (e.clientY - rect.top - pan.y) / zoom;

      const dx = worldX - resizeNodeStartMouse.x;
      const dy = worldY - resizeNodeStartMouse.y;

      const resizingNode = nodes.find(n => n.id === resizingNodeId);
      const isAspectLocked = resizingNode && (resizingNode.type === 'image-gen' || resizingNode.type === 'video-gen');

      let targetWidth = resizeNodeStartSize.width + dx;
      let targetHeight = resizeNodeStartSize.height + dy;

      if (isAspectLocked && resizingNode) {
        const numericRatio = getNumericAspectRatio(resizingNode.ratio || '1:1');
        
        if (resizeNodeDirection === 'h') {
          targetHeight = resizeNodeStartSize.height + dy;
          const maxHeight = 500;
          if (targetHeight > maxHeight) targetHeight = maxHeight;
          if (targetHeight < 120) targetHeight = 120;
          targetWidth = Math.round((targetHeight - 37) * numericRatio);
        } else {
          targetWidth = resizeNodeStartSize.width + dx;
          const maxWidth = 360;
          if (targetWidth > maxWidth) targetWidth = maxWidth;
          if (targetWidth < 160) targetWidth = 160;
          targetHeight = Math.round(targetWidth / numericRatio) + 37;
        }

        // Final bounds enforcement
        const maxWidth = 360;
        const maxHeight = 500;
        if (targetWidth > maxWidth) {
          targetWidth = maxWidth;
          targetHeight = Math.round(targetWidth / numericRatio) + 37;
        }
        if (targetWidth < 160) {
          targetWidth = 160;
          targetHeight = Math.round(targetWidth / numericRatio) + 37;
        }
        if (targetHeight > maxHeight) {
          targetHeight = maxHeight;
          targetWidth = Math.round((targetHeight - 37) * numericRatio);
        }
        if (targetHeight < 120) {
          targetHeight = 120;
          targetWidth = Math.round((targetHeight - 37) * numericRatio);
        }
        
        setAlignGuides([]); // Clear alignment guides for aspect locked resize
      } else {
        // Minimum bounds
        targetWidth = Math.max(160, targetWidth);
        targetHeight = Math.max(120, targetHeight);

        // Snap to Grid for Resizing
        if (snapToGrid) {
          targetWidth = Math.round(targetWidth / 16) * 16;
          targetHeight = Math.round(targetHeight / 16) * 16;
        }

        // Calculate resizing alignment guides and snap width/height to align with other nodes' boundaries!
        const snapResult = calculateResizeSnapping(resizingNodeId, targetWidth, targetHeight, nodes);
        targetWidth = snapResult.width;
        targetHeight = snapResult.height;
        setAlignGuides(snapResult.guides);
      }

      setNodes(prev => {
        const updatedNodes = prev.map(n => {
          if (n.id === resizingNodeId) {
            return {
              ...n,
              width: Math.round(targetWidth),
              height: Math.round(targetHeight)
            };
          }
          return n;
        });

        const targetNode = updatedNodes.find(n => n.id === resizingNodeId);
        if (targetNode && targetNode.groupId) {
          const gId = targetNode.groupId;
          setGroups(currentGroups => {
            return currentGroups.map(group => {
              if (group.id === gId) {
                const groupNodes = updatedNodes.filter(n => group.nodeIds.includes(n.id));
                if (groupNodes.length === 0) return group;

                let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
                groupNodes.forEach(n => {
                  minX = Math.min(minX, n.x);
                  minY = Math.min(minY, n.y);
                  maxX = Math.max(maxX, n.x + (n.width || 320));
                  maxY = Math.max(maxY, n.y + (n.height || 300));
                });

                const pad = 45;
                return {
                  ...group,
                  x: minX - pad,
                  y: minY - pad - 40,
                  width: maxX - minX + pad * 2,
                  height: maxY - minY + pad * 2 + 40
                };
              }
              return group;
            });
          });
        }

        return updatedNodes;
      });
      return;
    }

    // Dragging standard FlowCard & Multi-selections
    if (draggingNodeId) {
      updateDragNodePositions(e.clientX, e.clientY);
      startEdgePanLoop();
      return;
    }

    // Dragging Port Connector Wire (with Magnetic Snapping!)
    if (activePortDrag) {
      if (!containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const worldX = (e.clientX - rect.left - pan.x) / zoom;
      const worldY = (e.clientY - rect.top - pan.y) / zoom;

      // Find closest port on other nodes (bidirectional support)
      let closestNode: CanvasNode | null = null;
      let minDistance = 75; // 75px snapping range threshold

      nodes.forEach(node => {
        if (node.id === activePortDrag.nodeId) return; // Don't connect to self
        
        // If starting from an output port, look for input port (left edge: x + 0)
        // If starting from an input port, look for output port (right edge: x + width)
        const targetType = activePortDrag.type === 'output' ? 'input' : 'output';
        const nodeW = typeof node.width === 'number' && !isNaN(node.width) ? node.width : 260;
        const nodeH = node.isCollapsed ? 36 : (typeof node.height === 'number' && !isNaN(node.height) ? node.height : 250);
        const portX = node.x + (targetType === 'output' ? nodeW : 0);
        const portY = node.y + nodeH / 2;
        const dist = Math.hypot(worldX - portX, worldY - portY);
        
        if (dist < minDistance) {
          minDistance = dist;
          closestNode = node;
        }
      });

      if (closestNode) {
        // Snap directly to target port coordinates
        const targetType = activePortDrag.type === 'output' ? 'input' : 'output';
        const closestNodeW = typeof (closestNode as CanvasNode).width === 'number' && !isNaN((closestNode as CanvasNode).width!) ? (closestNode as CanvasNode).width! : 260;
        const closestNodeH = (closestNode as CanvasNode).isCollapsed ? 36 : (typeof (closestNode as CanvasNode).height === 'number' && !isNaN((closestNode as CanvasNode).height!) ? (closestNode as CanvasNode).height! : 250);
        const targetPortX = (closestNode as CanvasNode).x + (targetType === 'output' ? closestNodeW : 0);
        const targetPortY = (closestNode as CanvasNode).y + closestNodeH / 2;
        setPortDragCurrentPos({ x: targetPortX, y: targetPortY });
        setSnappingTargetNodeId((closestNode as CanvasNode).id);
      } else {
        // Smoothly draw to exact mouse coordinates
        setPortDragCurrentPos({ x: worldX, y: worldY });
        setSnappingTargetNodeId(null);
      }
      startEdgePanLoop();
      return;
    }

    // Updating Lasso Selection Box
    if (selectionBox) {
      if (!containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const worldX = (e.clientX - rect.left - pan.x) / zoom;
      const worldY = (e.clientY - rect.top - pan.y) / zoom;
      setSelectionBox(prev => prev ? { ...prev, currentX: worldX, currentY: worldY } : null);
    }
  };

  // Mouse Up Event on Canvas
  const handleMouseUp = (e?: React.MouseEvent) => {
    // Check if dragging or resizing actually changed node/group properties before stopping
    if (dragInitialStateRef.current) {
      const initial = dragInitialStateRef.current;
      let hasChanged = false;
      if (initial.nodes.length !== nodes.length) {
        hasChanged = true;
      } else {
        for (let i = 0; i < nodes.length; i++) {
          const nInit = initial.nodes.find(n => n.id === nodes[i].id);
          if (!nInit || nInit.x !== nodes[i].x || nInit.y !== nodes[i].y || nInit.width !== nodes[i].width || nInit.height !== nodes[i].height) {
            hasChanged = true;
            break;
          }
        }
      }
      
      if (!hasChanged) {
        if (initial.groups.length !== groups.length) {
          hasChanged = true;
        } else {
          for (let i = 0; i < groups.length; i++) {
            const gInit = initial.groups.find(g => g.id === groups[i].id);
            if (!gInit || gInit.x !== groups[i].x || gInit.y !== groups[i].y || gInit.width !== groups[i].width || gInit.height !== groups[i].height) {
              hasChanged = true;
              break;
            }
          }
        }
      }

      if (!hasChanged) {
        if (initial.connections.length !== connections.length) {
          hasChanged = true;
        } else {
          for (let i = 0; i < connections.length; i++) {
            const cInit = initial.connections.find(c => c.id === connections[i].id);
            if (!cInit || cInit.fromNodeId !== connections[i].fromNodeId || cInit.toNodeId !== connections[i].toNodeId) {
              hasChanged = true;
              break;
            }
          }
        }
      }

      if (hasChanged) {
        const snapshot = dragInitialStateRef.current;
        setHistory(prev => {
          const updated = [...prev, snapshot];
          if (updated.length > 30) updated.shift();
          return updated;
        });
        setRedoStack([]);
      }
      dragInitialStateRef.current = null;
    }

    setIsPanning(false);
    setDraggingGroupId(null);
    setResizingGroupId(null);
    setResizingNodeId(null);
    setDraggingConnectionId(null);
    dragGroupNodesOffsetsRef.current = {};

    // Stop auto-panning and clear guidelines immediately
    stopEdgePanLoop();
    setAlignGuides([]);

    // Node drag release -> Check if dropped inside a Group Frame (Snapping!)
    if (draggingNodeId) {
      const movedNodeIds = Object.keys(dragNodesOffsetsRef.current);

      setNodes(currentNodes => {
        let updatedNodes = [...currentNodes];

        setGroups(currentGroups => {
          let updatedGroups = [...currentGroups];

          movedNodeIds.forEach(nodeId => {
            const node = updatedNodes.find(n => n.id === nodeId);
            if (!node) return;

            const nodeCenterX = node.x + node.width / 2;
            const nodeCenterY = node.y + (node.height || 300) / 2;

            // Find if center is contained in any group
            const targetGroup = updatedGroups.find(g => {
              return nodeCenterX >= g.x &&
                     nodeCenterX <= g.x + g.width &&
                     nodeCenterY >= g.y &&
                     nodeCenterY <= g.y + g.height;
            });

            if (targetGroup) {
              // Connect to target group list if not present
              updatedGroups = updatedGroups.map(g => {
                if (g.id === targetGroup.id) {
                  if (!g.nodeIds.includes(nodeId)) {
                    return { ...g, nodeIds: [...g.nodeIds, nodeId] };
                  }
                } else {
                  return { ...g, nodeIds: g.nodeIds.filter(id => id !== nodeId) };
                }
                return g;
              });

              updatedNodes = updatedNodes.map(n => n.id === nodeId ? { ...n, groupId: targetGroup.id } : n);
            } else {
              // Outside of group: disband old group mapping
              if (node.groupId) {
                const oldGroupId = node.groupId;
                updatedGroups = updatedGroups.map(g => g.id === oldGroupId ? { ...g, nodeIds: g.nodeIds.filter(id => id !== nodeId) } : g);
                updatedNodes = updatedNodes.map(n => n.id === nodeId ? { ...n, groupId: undefined } : n);
              }
            }
          });

          // Adaptively recalculate bounds of all affected groups on node drop
          updatedGroups = updatedGroups.map(group => {
            const groupNodes = updatedNodes.filter(n => group.nodeIds.includes(n.id));
            if (groupNodes.length === 0) return group;

            let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
            groupNodes.forEach(n => {
              minX = Math.min(minX, n.x);
              minY = Math.min(minY, n.y);
              maxX = Math.max(maxX, n.x + (n.width || 320));
              maxY = Math.max(maxY, n.y + (n.height || 300));
            });

            const pad = 45;
            return {
              ...group,
              x: minX - pad,
              y: minY - pad - 40,
              width: maxX - minX + pad * 2,
              height: maxY - minY + pad * 2 + 40
            };
          });

          // Filter out completely empty custom groups
          return updatedGroups.filter(g => g.nodeIds.length > 0 || g.id === 'group-initial');
        });

        return updatedNodes;
      });

      setDraggingNodeId(null);
      dragNodesOffsetsRef.current = {};
    }

    // Connector drag release -> form workflow connections
    if (activePortDrag) {
      if (e) {
        const distanceMoved = Math.hypot(
          e.clientX - portDragStartMouseRef.current.x,
          e.clientY - portDragStartMouseRef.current.y
        );

        // If the mouse moved very little, assume a sticky click-to-connect click!
        if (distanceMoved < 6 && !isStickyConnection) {
          setIsStickyConnection(true);
          return; // Keep activePortDrag alive!
        }
      }

      const releaseX = portDragCurrentPos.x;
      const releaseY = portDragCurrentPos.y;

      const targetNode = nodes.find(node => {
        if (node.id === activePortDrag.nodeId) return false;
        
        // Find the matching target port location based on drag starting type
        const targetType = activePortDrag.type === 'output' ? 'input' : 'output';
        const targetNodeW = typeof node.width === 'number' && !isNaN(node.width) ? node.width : 260;
        const targetNodeH = node.isCollapsed ? 36 : (typeof node.height === 'number' && !isNaN(node.height) ? node.height : 250);
        const portX = node.x + (targetType === 'output' ? targetNodeW : 0);
        const portY = node.y + targetNodeH / 2;
        
        const distance = Math.hypot(releaseX - portX, releaseY - portY);
        return distance < 75; // Snapping drop tolerance
      });

      if (targetNode) {
        // Correctly orient the link: from output to input
        const fromId = activePortDrag.type === 'output' ? activePortDrag.nodeId : targetNode.id;
        const toId = activePortDrag.type === 'input' ? activePortDrag.nodeId : targetNode.id;

        const existingConn = connections.some(c => 
          c.fromNodeId === fromId && c.toNodeId === toId
        );

        if (!existingConn && fromId !== toId) { saveHistory();
          saveHistory();
          const newConnId = `conn-${Date.now()}`;
          setConnections(prev => [...prev, {
            id: newConnId,
            fromNodeId: fromId,
            toNodeId: toId
          }]);
        }
      }

      setActivePortDrag(null);
      setSnappingTargetNodeId(null);
      setIsStickyConnection(false);
    }

    // Selection box release -> Select nodes within rectangle
    if (selectionBox) {
      const x1 = Math.min(selectionBox.startX, selectionBox.currentX);
      const x2 = Math.max(selectionBox.startX, selectionBox.currentX);
      const y1 = Math.min(selectionBox.startY, selectionBox.currentY);
      const y2 = Math.max(selectionBox.startY, selectionBox.currentY);

      const selectedIds = nodes
        .filter(node => {
          const nodeCenterX = node.x + node.width / 2;
          const nodeCenterY = node.y + (node.height || 300) / 2;
          return nodeCenterX >= x1 && nodeCenterX <= x2 && nodeCenterY >= y1 && nodeCenterY <= y2;
        })
        .map(node => node.id);

      if (e && e.shiftKey) {
        setSelectedNodeIds(prev => Array.from(new Set([...prev, ...selectedIds])));
      } else {
        setSelectedNodeIds(selectedIds);
      }
      setSelectionBox(null);
    }

    // Defer selection toggles if we didn't drag
    if (!nodeDragMovedRef.current) {
      if (lastShiftDeselectionCandidateRef.current !== null) {
        const id = lastShiftDeselectionCandidateRef.current;
        setSelectedNodeIds(prev => prev.filter(nid => nid !== id));
      } else if (lastDeselectionCandidateRef.current !== null) {
        const id = lastDeselectionCandidateRef.current;
        setSelectedNodeIds([id]);
      }
    }

    // Reset drag and selection candidate states
    nodeDragMovedRef.current = false;
    lastShiftDeselectionCandidateRef.current = null;
    lastDeselectionCandidateRef.current = null;
    nextSelectedNodeIdsRef.current = null;
  };

  // Node Drag Initiated
  const handleNodeDragStart = (id: string, e: React.MouseEvent) => {
    if (activePortDrag) return;
    if (!containerRef.current) return;
    const node = nodes.find(n => n.id === id);
    if (!node) return;

    // Save history prior to change (Undo Queue)
    dragInitialStateRef.current = { nodes, groups, connections };

    const rect = containerRef.current.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const worldMouseX = (mouseX - pan.x) / zoom;
    const worldMouseY = (mouseY - pan.y) / zoom;

    // Initialize drag movement detection
    nodeDragMovedRef.current = false;

    // Define multi-dragging subset (either current multi-selection or just the single clicked card)
    let activeSelection = nextSelectedNodeIdsRef.current !== null ? nextSelectedNodeIdsRef.current : selectedNodeIds;
    nextSelectedNodeIdsRef.current = null;

    if (!activeSelection.includes(id)) {
      activeSelection = [id];
      setSelectedNodeIds([id]);
      setSelectedConnectionId(null);
      setSelectedGroupId(null);
      lastShiftDeselectionCandidateRef.current = null;
      lastDeselectionCandidateRef.current = null;
    }

    // Record individual offsets relative to mouse positions
    const offsets: Record<string, { x: number; y: number }> = {};
    activeSelection.forEach(nid => {
      const n = nodes.find(item => item.id === nid);
      if (n) {
        offsets[nid] = {
          x: worldMouseX - n.x,
          y: worldMouseY - n.y
        };
      }
    });
    dragNodesOffsetsRef.current = offsets;

    setDraggingNodeId(id);
    setDragOffset({
      x: worldMouseX - node.x,
      y: worldMouseY - node.y
    });

    mouseScreenPosRef.current = { x: e.clientX, y: e.clientY };
    startEdgePanLoop();
  };

  // Port Drag Initiated
  const handlePortMouseDown = (id: string, type: 'input' | 'output', e: React.MouseEvent) => {
    e.stopPropagation();

    // If a sticky connection is active from another node, clicking any port completes the connection!
    if (activePortDrag && activePortDrag.nodeId !== id) {
      const fromId = activePortDrag.type === 'output' ? activePortDrag.nodeId : id;
      const toId = activePortDrag.type === 'input' ? activePortDrag.nodeId : id;

      const existingConn = connections.some(c => 
        c.fromNodeId === fromId && c.toNodeId === toId
      );

      if (!existingConn && fromId !== toId) { saveHistory();
        setConnections(prev => [...prev, {
          id: `conn-${Date.now()}`,
          fromNodeId: fromId,
          toNodeId: toId
        }]);
      }

      setActivePortDrag(null);
      setSnappingTargetNodeId(null);
      setIsStickyConnection(false);
      return;
    }

    const node = nodes.find(n => n.id === id);
    if (!node) return;

    const nodeW = typeof node.width === 'number' && !isNaN(node.width) ? node.width : 260;
    const nodeH = node.isCollapsed ? 36 : (typeof node.height === 'number' && !isNaN(node.height) ? node.height : 250);
    const portX = type === 'output' ? (node.x + nodeW) : node.x;
    const portY = node.y + nodeH / 2;

    setActivePortDrag({
      nodeId: id,
      type,
      startX: portX,
      startY: portY
    });
    setPortDragCurrentPos({ x: portX, y: portY });
    portDragStartMouseRef.current = { x: e.clientX, y: e.clientY };
    setIsStickyConnection(false);
  };

  // Group Drag Initiated
  const handleGroupDragStart = (groupId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    
    // Don't drag group if interacting with inputs, buttons or pickers
    const target = e.target as HTMLElement;
    if (target.closest('button') || target.closest('input') || target.closest('.color-picker-dot')) return;

    dragInitialStateRef.current = { nodes, groups, connections };
    setSelectedGroupId(groupId);
    setSelectedNodeIds([]); // clear general card selections
    setSelectedConnectionId(null);

    if (!containerRef.current) return;
    const group = groups.find(g => g.id === groupId);
    if (!group) return;

    const rect = containerRef.current.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const worldX = (mouseX - pan.x) / zoom;
    const worldY = (mouseY - pan.y) / zoom;

    setDraggingGroupId(groupId);
    setDragGroupOffset({
      x: worldX - group.x,
      y: worldY - group.y
    });

    // Populate offsets of nested nodes inside this group relative to group coordinates
    const offsets: Record<string, { x: number; y: number }> = {};
    group.nodeIds.forEach(nodeId => {
      const node = nodes.find(n => n.id === nodeId);
      if (node) {
        offsets[nodeId] = {
          x: node.x - group.x,
          y: node.y - group.y
        };
      }
    });
    dragGroupNodesOffsetsRef.current = offsets;
  };

  // Group Resize Initiated
  const handleGroupResizeStart = (groupId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    e.preventDefault();
    if (!containerRef.current) return;
    const group = groups.find(g => g.id === groupId);
    if (!group) return;

    dragInitialStateRef.current = { nodes, groups, connections };
    const rect = containerRef.current.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const worldX = (mouseX - pan.x) / zoom;
    const worldY = (mouseY - pan.y) / zoom;

    setResizingGroupId(groupId);
    setResizeStartSize({ width: group.width, height: group.height });
    setResizeStartMouse({ x: worldX, y: worldY });
  };

  // Node Resize Initiated
  const handleNodeResizeStart = (nodeId: string, e: React.MouseEvent, direction: 'w' | 'h' | 'both' = 'both') => {
    e.stopPropagation();
    e.preventDefault();
    if (!containerRef.current) return;
    const node = nodes.find(n => n.id === nodeId);
    if (!node) return;

    dragInitialStateRef.current = { nodes, groups, connections };
    const rect = containerRef.current.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const worldX = (mouseX - pan.x) / zoom;
    const worldY = (mouseY - pan.y) / zoom;

    setResizingNodeId(nodeId);
    setResizeNodeDirection(direction);
    setResizeNodeStartSize({ 
      width: node.width || 320, 
      height: node.height || 300 
    });
    setResizeNodeStartMouse({ x: worldX, y: worldY });
  };

  // Node modifications callback
  const handleUpdateNode = useCallback((id: string, updates: Partial<CanvasNode>) => {
    const isTextUpdate = 'title' in updates || 'content' in updates || 'prompt' in updates;
    
    if (isTextUpdate) {
      const now = Date.now();
      const lastSave = lastTextSaveRef.current[id] || 0;
      if (now - lastSave > 3000) {
        saveHistory();
        lastTextSaveRef.current[id] = now;
      }
    } else {
      saveHistory();
    }

    setNodes(prev => prev.map(n => {
      if (n.id === id) {
        const merged = { ...n, ...updates };
        if (updates.ratio && (n.type === 'image-gen' || n.type === 'video-gen')) {
          // Keep current width, adapt height based on new ratio
          merged.height = getAdaptedHeight(n.type, n.width, updates.ratio);
        }
        return merged;
      }
      return n;
    }));
  }, [saveHistory]);

  const triggerNodeGeneration = (id: string, customPrompt?: string, customSettings?: any) => {
    const node = nodes.find(n => n.id === id);
    if (!node || node.status === 'generating') return;

    saveHistory();
    const finalPrompt = customPrompt !== undefined ? customPrompt : (node.prompt || '');
    const finalSettings = customSettings || {};

    // Filter out visual/layout sizes to prevent node box distortion
    const { width, height, imageWidth, imageHeight, ...cleanSettings } = finalSettings;

    const updates: Partial<CanvasNode> = { 
      status: 'generating', 
      progress: 5, 
      mediaUrl: undefined,
      prompt: finalPrompt,
      ...cleanSettings
    };

    handleUpdateNode(id, updates);

    if (node.type === 'text') {
      setTimeout(() => {
        setNodes(prev => prev.map(n => n.id === id ? {
          ...n,
          status: 'idle',
          progress: 100,
          content: `${n.content || ''}\n\n[AI]: 这里是根据提示词 "${finalPrompt}" 生成的文本内容。`.trim()
        } : n));
      }, 1000);
    } else if (node.type === 'image-gen') {
      CanvasService.generateImage(finalPrompt, finalSettings.ratio || node.ratio || '1:1', (p, msg) => {
        setNodes(prev => prev.map(n => n.id === id ? { ...n, progress: p, content: msg } : n));
      }, finalSettings.model || node.model).then((url) => {
        setNodes(prev => prev.map(n => n.id === id ? { ...n, status: 'completed', progress: 100, mediaUrl: url } : n));
      }).catch((error: unknown) => {
        const message = error instanceof Error ? error.message : '图片生成失败';
        setNodes(prev => prev.map(n => n.id === id ? { ...n, status: 'failed', progress: 0 } : n));
        showToastMessage(message, 'error');
      });
    } else if (node.type === 'video-gen') {
      CanvasService.generateVideo(finalPrompt, (p, msg) => {
        setNodes(prev => prev.map(n => n.id === id ? { ...n, progress: p, content: msg } : n));
      }, finalSettings.model || node.model).then((url) => {
        setNodes(prev => prev.map(n => n.id === id ? { ...n, status: 'completed', progress: 100, mediaUrl: url } : n));
      }).catch((error: unknown) => {
        const message = error instanceof Error ? error.message : '视频生成失败';
        setNodes(prev => prev.map(n => n.id === id ? { ...n, status: 'failed', progress: 0 } : n));
        showToastMessage(message, 'error');
      });
    }
  };

  const handleInputPromptChange = (val: string) => {
    if (selectedNodeIds.length === 1) {
      const selectedId = selectedNodeIds[0];
      handleUpdateNode(selectedId, { prompt: val });
    }
  };

  const handleInputSettingsChange = (settings: any) => {
    if (selectedNodeIds.length === 1) {
      const selectedId = selectedNodeIds[0];
      // Filter out visual layout sizes to prevent node box distortion
      const { width, height, imageWidth, imageHeight, ...cleanSettings } = settings || {};
      handleUpdateNode(selectedId, cleanSettings);
    }
  };

  const handleInputModeChange = (mode: string) => {
    if (selectedNodeIds.length === 1) {
      const selectedId = selectedNodeIds[0];
      const node = nodes.find(n => n.id === selectedId);
      if (!node) return;
      
      const targetType = mode === 'image' ? 'image-gen' : mode === 'video' ? 'video-gen' : 'text';
      if (node.type !== targetType) {
        const titles = {
          'text': '创意大纲草稿',
          'image-gen': 'AI 创意图源',
          'video-gen': 'AI 镜头渲染'
        };
        handleUpdateNode(selectedId, {
          type: targetType,
          width: targetType === 'text' ? 320 : 260,
          height: targetType === 'text' ? 250 : targetType === 'image-gen' ? 280 : 190,
          title: `${titles[targetType]} #${nodes.length + 1}`
        });
      }
    }
  };

  const handleDeleteNode = (id: string) => {
    saveHistory();
    setNodes(prev => prev.filter(n => n.id !== id));
    setConnections(prev => prev.filter(c => c.fromNodeId !== id && c.toNodeId !== id));
    
    // Remove from group lists
    setGroups(prev => prev.map(g => ({
      ...g,
      nodeIds: g.nodeIds.filter(nodeId => nodeId !== id)
    })));

    setSelectedNodeIds(prev => prev.filter(nid => nid !== id));
  };

  const handleSelectNode = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    
    // If a sticky connection is active from another node, clicking this card completes the connection!
    if (activePortDrag && activePortDrag.nodeId !== id) {
      const fromId = activePortDrag.type === 'output' ? activePortDrag.nodeId : id;
      const toId = activePortDrag.type === 'input' ? activePortDrag.nodeId : id;

      const existingConn = connections.some(c => 
        c.fromNodeId === fromId && c.toNodeId === toId
      );

      if (!existingConn && fromId !== toId) { saveHistory();
        setConnections(prev => [...prev, {
          id: `conn-${Date.now()}`,
          fromNodeId: fromId,
          toNodeId: toId
        }]);
      }

      setActivePortDrag(null);
      setSnappingTargetNodeId(null);
      setIsStickyConnection(false);
      return;
    }
    
    // Reset candidate references
    lastShiftDeselectionCandidateRef.current = null;
    lastDeselectionCandidateRef.current = null;

    let nextSelection = [...selectedNodeIds];

    if (e.shiftKey) {
      if (selectedNodeIds.includes(id)) {
        // Already selected: do NOT deselect immediately to allow group dragging. Defer to MouseUp.
        lastShiftDeselectionCandidateRef.current = id;
      } else {
        // Not selected: add to selection
        nextSelection = [...selectedNodeIds, id];
      }
    } else {
      if (selectedNodeIds.includes(id)) {
        // Already selected: do NOT clear other selections immediately to allow group dragging. Defer to MouseUp.
        lastDeselectionCandidateRef.current = id;
      } else {
        // Clicked a new node: clear other selections
        nextSelection = [id];
      }
    }

    nextSelectedNodeIdsRef.current = nextSelection;
    setSelectedNodeIds(nextSelection);
    setSelectedConnectionId(null);
    setSelectedGroupId(null);
  };

  // Tool / Node additions
  const handleAddNode = (type: 'text' | 'image-gen' | 'video-gen' | 'sticky') => {
    if (!containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    
    const centerWorldX = (rect.width / 2 - pan.x) / zoom;
    const centerWorldY = (rect.height / 2 - pan.y) / zoom;

    const newNodeId = `node-${Date.now()}`;
    const titles = {
      'text': '创意大纲草稿',
      'image-gen': 'AI 创意图源',
      'video-gen': 'AI 镜头渲染',
      'sticky': '便签注释'
    };

    const newNode: CanvasNode = {
      id: newNodeId,
      type,
      x: Math.round(centerWorldX - (type === 'sticky' ? 120 : (type === 'image-gen' || type === 'video-gen' ? 130 : 160))),
      y: Math.round(centerWorldY - (type === 'sticky' ? 100 : (type === 'image-gen' || type === 'video-gen' ? 150 : 150))),
      width: type === 'sticky' ? 240 : (type === 'image-gen' || type === 'video-gen' ? 260 : 320),
      height: type === 'sticky' ? 200 : type === 'text' ? 250 : type === 'image-gen' ? 280 : 190,
      title: `${titles[type]} #${nodes.length + 1}`,
      status: 'idle',
      color: type === 'sticky' ? 'yellow' : undefined,
      content: type === 'sticky' ? '' : undefined
    };

    saveHistory();
    setNodes(prev => [...prev, newNode]);
    setSelectedNodeIds([newNodeId]);
  };

  // Group creation from lasso selections (框选分组)
  const handleCreateGroupFromSelection = () => {
    if (selectedNodeIds.length === 0) return;
    const selectedNodes = nodes.filter(n => selectedNodeIds.includes(n.id));
    if (selectedNodes.length === 0) return;

    let minX = Infinity;
    let maxX = -Infinity;
    let minY = Infinity;
    let maxY = -Infinity;

    selectedNodes.forEach(n => {
      if (n.x < minX) minX = n.x;
      if (n.x + n.width > maxX) maxX = n.x + n.width;
      if (n.y < minY) minY = n.y;
      if (n.y + (n.height || 300) > maxY) maxY = n.y + (n.height || 300);
    });

    const padding = 45;
    const newGroupId = `group-${Date.now()}`;
    const newGroup: CanvasGroup = {
      id: newGroupId,
      title: `自定义分组 #${groups.length + 1}`,
      color: ['cyan', 'yellow', 'pink', 'emerald', 'violet', 'orange'][groups.length % 6] as any,
      x: minX - padding,
      y: minY - padding - 40,
      width: (maxX - minX) + padding * 2,
      height: (maxY - minY) + padding * 2 + 40,
      nodeIds: [...selectedNodeIds]
    };

    saveHistory();
    // Update node links
    setNodes(prev => prev.map(n => {
      if (selectedNodeIds.includes(n.id)) {
        return { ...n, groupId: newGroupId };
      }
      return n;
    }));

    // Clean up old groups' node references and remove empty groups
    setGroups(prev => {
      const updatedGroups = prev.map(g => ({
        ...g,
        nodeIds: g.nodeIds.filter(id => !selectedNodeIds.includes(id))
      })).filter(g => g.nodeIds.length > 0 || g.id === 'group-initial');
      return [...updatedGroups, newGroup];
    });

    setSelectedNodeIds([]); // clear selection
    showToastMessage('成功创建新分组容器', 'success');
  };

  // Batch delete selected cards
  const handleBatchDelete = () => {
    if (selectedNodeIds.length === 0) return;
    saveHistory();
    setNodes(prev => prev.filter(n => !selectedNodeIds.includes(n.id)));
    setConnections(prev => prev.filter(c => !selectedNodeIds.includes(c.fromNodeId) && !selectedNodeIds.includes(c.toNodeId)));
    
    // Clean group lists
    setGroups(prev => prev.map(g => ({
      ...g,
      nodeIds: g.nodeIds.filter(id => !selectedNodeIds.includes(id))
    })).filter(g => g.nodeIds.length > 0 || g.id === 'group-initial'));

    setSelectedNodeIds([]);
  };

  // Group disband handles
  const handleDisbandGroup = (groupId: string) => {
    saveHistory();
    setGroups(prev => prev.filter(g => g.id !== groupId));
    setNodes(prev => prev.map(n => n.groupId === groupId ? { ...n, groupId: undefined } : n));
  };

  const handleAlignNodes = useCallback((alignment: 'left' | 'center-h' | 'right' | 'top' | 'center-v' | 'bottom' | 'distribute-h' | 'distribute-v') => {
    if (selectedNodeIds.length < 2) return;
    
    saveHistory();
    
    setNodes(currentNodes => {
      const selectedNodes = currentNodes.filter(n => selectedNodeIds.includes(n.id));
      if (selectedNodes.length < 2) return currentNodes;

      let updatedNodes = [...currentNodes];
      const affectedGroupIds = new Set<string>();

      if (alignment === 'left') {
        const minX = Math.min(...selectedNodes.map(n => n.x));
        updatedNodes = currentNodes.map(n => {
          if (selectedNodeIds.includes(n.id)) {
            if (n.groupId) affectedGroupIds.add(n.groupId);
            return { ...n, x: minX };
          }
          return n;
        });
      } else if (alignment === 'center-h') {
        const avgCenter = selectedNodes.reduce((sum, n) => sum + (n.x + (n.width || 320) / 2), 0) / selectedNodes.length;
        updatedNodes = currentNodes.map(n => {
          if (selectedNodeIds.includes(n.id)) {
            if (n.groupId) affectedGroupIds.add(n.groupId);
            return { ...n, x: Math.round(avgCenter - (n.width || 320) / 2) };
          }
          return n;
        });
      } else if (alignment === 'right') {
        const maxRight = Math.max(...selectedNodes.map(n => n.x + (n.width || 320)));
        updatedNodes = currentNodes.map(n => {
          if (selectedNodeIds.includes(n.id)) {
            if (n.groupId) affectedGroupIds.add(n.groupId);
            return { ...n, x: maxRight - (n.width || 320) };
          }
          return n;
        });
      } else if (alignment === 'top') {
        const minY = Math.min(...selectedNodes.map(n => n.y));
        updatedNodes = currentNodes.map(n => {
          if (selectedNodeIds.includes(n.id)) {
            if (n.groupId) affectedGroupIds.add(n.groupId);
            return { ...n, y: minY };
          }
          return n;
        });
      } else if (alignment === 'center-v') {
        const avgCenter = selectedNodes.reduce((sum, n) => sum + (n.y + (n.height || 250) / 2), 0) / selectedNodes.length;
        updatedNodes = currentNodes.map(n => {
          if (selectedNodeIds.includes(n.id)) {
            if (n.groupId) affectedGroupIds.add(n.groupId);
            return { ...n, y: Math.round(avgCenter - (n.height || 250) / 2) };
          }
          return n;
        });
      } else if (alignment === 'bottom') {
        const maxBottom = Math.max(...selectedNodes.map(n => n.y + (n.height || 250)));
        updatedNodes = currentNodes.map(n => {
          if (selectedNodeIds.includes(n.id)) {
            if (n.groupId) affectedGroupIds.add(n.groupId);
            return { ...n, y: maxBottom - (n.height || 250) };
          }
          return n;
        });
      } else if (alignment === 'distribute-h') {
        const sorted = [...selectedNodes].sort((a, b) => a.x - b.x);
        const minX = sorted[0].x;
        const lastNode = sorted[sorted.length - 1];
        const maxX = lastNode.x + (lastNode.width || 320);
        const sumWidths = sorted.reduce((sum, n) => sum + (n.width || 320), 0);
        const totalGap = (maxX - minX) - sumWidths;
        const gap = sorted.length > 1 ? totalGap / (sorted.length - 1) : 0;
        
        let currentX = minX;
        const newPositions = new Map<string, number>();
        sorted.forEach(node => {
          newPositions.set(node.id, Math.round(currentX));
          currentX += (node.width || 320) + gap;
        });

        updatedNodes = currentNodes.map(n => {
          if (selectedNodeIds.includes(n.id)) {
            if (n.groupId) affectedGroupIds.add(n.groupId);
            return { ...n, x: newPositions.get(n.id) ?? n.x };
          }
          return n;
        });
      } else if (alignment === 'distribute-v') {
        const sorted = [...selectedNodes].sort((a, b) => a.y - b.y);
        const minY = sorted[0].y;
        const lastNode = sorted[sorted.length - 1];
        const maxY = lastNode.y + (lastNode.height || 250);
        const sumHeights = sorted.reduce((sum, n) => sum + (n.height || 250), 0);
        const totalGap = (maxY - minY) - sumHeights;
        const gap = sorted.length > 1 ? totalGap / (sorted.length - 1) : 0;
        
        let currentY = minY;
        const newPositions = new Map<string, number>();
        sorted.forEach(node => {
          newPositions.set(node.id, Math.round(currentY));
          currentY += (node.height || 250) + gap;
        });

        updatedNodes = currentNodes.map(n => {
          if (selectedNodeIds.includes(n.id)) {
            if (n.groupId) affectedGroupIds.add(n.groupId);
            return { ...n, y: newPositions.get(n.id) ?? n.y };
          }
          return n;
        });
      }

      // If any of the aligned nodes belong to groups, recalculate their bounds
      if (affectedGroupIds.size > 0) {
        setGroups(currentGroups => {
          return currentGroups.map(group => {
            if (affectedGroupIds.has(group.id)) {
              const groupNodes = updatedNodes.filter(n => group.nodeIds.includes(n.id));
              if (groupNodes.length === 0) return group;

              let gMinX = Infinity, gMinY = Infinity, gMaxX = -Infinity, gMaxY = -Infinity;
              groupNodes.forEach(n => {
                gMinX = Math.min(gMinX, n.x);
                gMinY = Math.min(gMinY, n.y);
                gMaxX = Math.max(gMaxX, n.x + (n.width || 320));
                gMaxY = Math.max(gMaxY, n.y + (n.height || 300));
              });

              const pad = 45;
              return {
                ...group,
                x: gMinX - pad,
                y: gMinY - pad - 40,
                width: gMaxX - gMinX + pad * 2,
                height: gMaxY - gMinY + pad * 2 + 40
              };
            }
            return group;
          });
        });
      }

      return updatedNodes;
    });
  }, [selectedNodeIds, saveHistory, setGroups, setNodes]);

  const handleToggleGroupCollapse = useCallback((groupId: string) => {
    saveHistory();
    setGroups(prev => prev.map(g => g.id === groupId ? { ...g, isCollapsed: !g.isCollapsed } : g));
  }, [saveHistory]);

  const handleUpdateGroupTitle = (groupId: string, title: string) => {
    saveHistory();
    setGroups(prev => prev.map(g => g.id === groupId ? { ...g, title } : g));
  };

  const handleUpdateGroupColor = (groupId: string, color: CanvasGroup['color']) => {
    saveHistory();
    setGroups(prev => prev.map(g => g.id === groupId ? { ...g, color } : g));
  };

  const handleResetView = () => {
    if (nodes.length === 0 && groups.length === 0) {
      setSmoothView({ x: 50, y: 50 }, 1);
      return;
    }

    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;

    nodes.forEach(node => {
      minX = Math.min(minX, node.x);
      minY = Math.min(minY, node.y);
      maxX = Math.max(maxX, node.x + (node.width || 320));
      maxY = Math.max(maxY, node.y + (node.height || 300));
    });

    groups.forEach(group => {
      minX = Math.min(minX, group.x);
      minY = Math.min(minY, group.y);
      maxX = Math.max(maxX, group.x + (group.width || 400));
      maxY = Math.max(maxY, group.y + (group.height || 300));
    });

    if (minX === Infinity) return;

    if (!containerRef.current) return;
    const containerWidth = containerRef.current.clientWidth;
    const containerHeight = containerRef.current.clientHeight;

    const padding = 100;
    const contentWidth = maxX - minX;
    const contentHeight = maxY - minY;

    const scaleX = (containerWidth - padding * 2) / contentWidth;
    const scaleY = (containerHeight - padding * 2) / contentHeight;
    let newZoom = Math.min(scaleX, scaleY, 2); // Max zoom 2x
    newZoom = Math.max(0.1, newZoom); // Min zoom 0.1x

    const contentCenterX = minX + contentWidth / 2;
    const contentCenterY = minY + contentHeight / 2;

    setSmoothView(
      {
        x: containerWidth / 2 - contentCenterX * newZoom,
        y: containerHeight / 2 - contentCenterY * newZoom
      },
      Number(newZoom.toFixed(2))
    );
  };

  const handleAutoLayout = (mode: 'hierarchy' | 'grid' = 'hierarchy') => {
    saveHistory();

    const START_X = 100;
    const START_Y = 100;

    let updatedNodes: CanvasNode[] = [];

    if (mode === 'grid') {
      // 1. GRID MATRIX LAYOUT
      // Sort nodes by current Y then X to preserve general layout order
      const sortedNodes = [...nodes].sort((a, b) => {
        if (Math.abs(a.y - b.y) < 100) {
          return a.x - b.x;
        }
        return a.y - b.y;
      });

      const cols = nodes.length <= 3 ? nodes.length : nodes.length <= 8 ? 3 : 4;
      const GAP_X = 360;
      const GAP_Y = 420;

      updatedNodes = nodes.map(node => {
        const index = sortedNodes.findIndex(n => n.id === node.id);
        const col = index % cols;
        const row = Math.floor(index / cols);
        return {
          ...node,
          x: Math.round(START_X + col * GAP_X),
          y: Math.round(START_Y + row * GAP_Y)
        };
      });
    } else {
      // 2. SMART HIERARCHICAL TREE LAYOUT
      const PADDING_X = 400;
      const PADDING_Y = 450;

      // Identify connected vs unconnected nodes
      const connectedNodeIds = new Set<string>();
      connections.forEach(c => {
        connectedNodeIds.add(c.fromNodeId);
        connectedNodeIds.add(c.toNodeId);
      });

      const connected = nodes.filter(n => connectedNodeIds.has(n.id));
      const unconnected = nodes.filter(n => !connectedNodeIds.has(n.id));

      if (connected.length === 0) {
        // Fallback to grid layout if there are no connections
        const cols = nodes.length <= 3 ? nodes.length : nodes.length <= 8 ? 3 : 4;
        const GAP_X = 360;
        const GAP_Y = 420;
        updatedNodes = nodes.map((node, index) => {
          const col = index % cols;
          const row = Math.floor(index / cols);
          return {
            ...node,
            x: Math.round(START_X + col * GAP_X),
            y: Math.round(START_Y + row * GAP_Y)
          };
        });
      } else {
        // Build adjacency list for connected nodes
        const incoming: Record<string, string[]> = {};
        connected.forEach(n => incoming[n.id] = []);

        connections.forEach(c => {
          if (incoming[c.toNodeId]) incoming[c.toNodeId].push(c.fromNodeId);
        });

        // Determine levels (longest path from root) for connected nodes
        const levels: Record<string, number> = {};
        connected.forEach(n => levels[n.id] = 0);

        let changed = true;
        let iterations = 0;
        while (changed && iterations < connected.length) {
          changed = false;
          connections.forEach(c => {
            if (levels[c.toNodeId] !== undefined && levels[c.fromNodeId] !== undefined) {
              if (levels[c.toNodeId] < levels[c.fromNodeId] + 1) {
                levels[c.toNodeId] = levels[c.fromNodeId] + 1;
                changed = true;
              }
            }
          });
          iterations++;
        }

        // Group connected nodes by level
        const levelGroups: Record<number, CanvasNode[]> = {};
        connected.forEach(n => {
          const lvl = levels[n.id] || 0;
          if (!levelGroups[lvl]) levelGroups[lvl] = [];
          levelGroups[lvl].push(n);
        });

        // Position connected nodes
        const connectedMapped = connected.map(node => {
          const lvl = levels[node.id] || 0;
          const nodesInLevel = levelGroups[lvl];
          const index = nodesInLevel.findIndex(n => n.id === node.id);
          return {
            ...node,
            x: Math.round(START_X + lvl * PADDING_X),
            y: Math.round(START_Y + index * PADDING_Y)
          };
        });

        // Determine maximum level (tree width)
        let maxLvl = 0;
        Object.keys(levelGroups).forEach(k => {
          const num = parseInt(k, 10);
          if (num > maxLvl) maxLvl = num;
        });

        // Position unconnected nodes to the right of the tree in a grid
        const unconnectedStartX = START_X + (maxLvl + 1) * PADDING_X + 80;
        const unconnectedCols = 2;
        const unconnectedGapX = 280;
        const unconnectedGapY = 320;

        const unconnectedMapped = unconnected.map((node, index) => {
          const col = index % unconnectedCols;
          const row = Math.floor(index / unconnectedCols);
          return {
            ...node,
            x: Math.round(unconnectedStartX + col * unconnectedGapX),
            y: Math.round(START_Y + row * unconnectedGapY)
          };
        });

        // Merge back
        const nodeMap = new Map<string, CanvasNode>();
        connectedMapped.forEach(n => nodeMap.set(n.id, n));
        unconnectedMapped.forEach(n => nodeMap.set(n.id, n));

        updatedNodes = nodes.map(n => nodeMap.get(n.id) || n);
      }
    }

    // Update group bounds based on their nodes
    const updatedGroups = groups.map(group => {
      const groupNodes = updatedNodes.filter(n => group.nodeIds.includes(n.id));
      if (groupNodes.length === 0) return group;

      let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
      groupNodes.forEach(n => {
        minX = Math.min(minX, n.x);
        minY = Math.min(minY, n.y);
        maxX = Math.max(maxX, n.x + (n.width || 320));
        maxY = Math.max(maxY, n.y + (n.height || 300));
      });

      const pad = 45;
      return {
        ...group,
        x: minX - pad,
        y: minY - pad - 40,
        width: maxX - minX + pad * 2,
        height: maxY - minY + pad * 2 + 40
      };
    });

    setNodes(updatedNodes);
    setGroups(updatedGroups);

    // Briefly delay to let the layout apply before resetting view
    setTimeout(() => {
      handleResetView();
    }, 100);

    showToastMessage(mode === 'grid' ? '已完成网格矩阵对齐' : '已完成树状分层排版', 'success');
  };

  const showToastMessage = (message: string, type: 'success' | 'error' | 'info' | 'loading' = 'success', duration = 3000) => {
    setToast({ message, type });
    if (type !== 'loading') {
      setTimeout(() => {
        setToast(prev => prev?.message === message ? null : prev);
      }, duration);
    }
  };

  const clearToast = () => setToast(null);

  const handleExport = () => {
    try {
      const dataStr = JSON.stringify({ nodes, groups, connections }, null, 2);
      const blob = new Blob([dataStr], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `canvas-export-${new Date().toISOString().slice(0, 10)}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      showToastMessage('JSON 流程导出成功！', 'success');
    } catch (err) {
      showToastMessage('导出 JSON 失败，请重试', 'error');
    }
  };

  const handleExportPNG = async () => {
    if (!containerRef.current) return;
    if (nodes.length === 0) {
      showToastMessage('画布上没有可以导出的节点', 'error');
      return;
    }

    showToastMessage('正在生成高清 PNG 图片...', 'loading');

    try {
      const { toPng } = await import('html-to-image');
      const dataUrl = await toPng(containerRef.current, {
        cacheBust: true,
        backgroundColor: '#0d0d0e',
        filter: (node: any) => {
          if (node.classList && node.classList.contains('no-export')) {
            return false;
          }
          return true;
        },
        style: {
          backgroundImage: `radial-gradient(circle at 1px 1px, rgba(255, 255, 255, 0.08) 1.5px, transparent 0)`,
          backgroundSize: `${32 * zoom}px ${32 * zoom}px`,
          backgroundPosition: `${pan.x}px ${pan.y}px`
        }
      });

      const a = document.createElement('a');
      a.href = dataUrl;
      a.download = `canvas-export-${new Date().toISOString().slice(0, 10)}.png`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      showToastMessage('PNG 图片导出成功！', 'success');
    } catch (err) {
      console.error('导出 PNG 失败', err);
      showToastMessage('导出 PNG 失败，请重试', 'error');
    }
  };

  const handleExportPDF = async () => {
    await exportCanvasToPDF({
      nodes,
      groups,
      connections,
      containerElement: containerRef.current,
      zoom,
      pan,
      showToast: showToastMessage
    });
  };

  const handleRestoreSnapshot = (snapshotData: {
    nodes: CanvasNode[];
    groups: CanvasGroup[];
    connections: Connection[];
    pan: { x: number; y: number };
    zoom: number;
  }) => {
    saveHistory();
    setNodes(snapshotData.nodes);
    setGroups(snapshotData.groups);
    setConnections(snapshotData.connections);
    setPan(snapshotData.pan);
    setZoom(snapshotData.zoom);
  };

  const handleLoadTemplate = (templateData: {
    nodes: CanvasNode[];
    groups: CanvasGroup[];
    connections: Connection[];
  }, mode: 'append' | 'replace') => {
    saveHistory();

    if (mode === 'replace') {
      setNodes(templateData.nodes);
      setGroups(templateData.groups);
      setConnections(templateData.connections);
      setPan({ x: 50, y: 50 });
      setZoom(1);
      showToastMessage('成功载入并重置画布模板', 'success');
      return;
    }

    // Append mode - avoid ID conflicts and center on current view
    const idMap: { [oldId: string]: string } = {};
    const groupIdMap: { [oldId: string]: string } = {};

    let minX = Infinity;
    let minY = Infinity;
    templateData.nodes.forEach(n => {
      if (n.x < minX) minX = n.x;
      if (n.y < minY) minY = n.y;
    });

    const viewportCenterX = (window.innerWidth / 2 - pan.x) / zoom;
    const viewportCenterY = (window.innerHeight / 2 - pan.y) / zoom;
    const offsetX = isFinite(minX) ? (viewportCenterX - minX - 150) : 100;
    const offsetY = isFinite(minY) ? (viewportCenterY - minY - 150) : 100;

    const newNodes = templateData.nodes.map(node => {
      const newId = `node-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`;
      idMap[node.id] = newId;
      return {
        ...node,
        id: newId,
        x: node.x + offsetX,
        y: node.y + offsetY,
        status: 'idle' as const
      };
    });

    const newGroups = templateData.groups.map(group => {
      const newGroupId = `group-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`;
      groupIdMap[group.id] = newGroupId;
      const newNodeIds = group.nodeIds.map(oldNodeId => idMap[oldNodeId]).filter(Boolean);
      return {
        ...group,
        id: newGroupId,
        x: group.x + offsetX,
        y: group.y + offsetY,
        nodeIds: newNodeIds
      };
    });

    const finalNodes = newNodes.map(node => {
      const oldNode = templateData.nodes.find(n => idMap[n.id] === node.id);
      if (oldNode?.groupId && groupIdMap[oldNode.groupId]) {
        return {
          ...node,
          groupId: groupIdMap[oldNode.groupId]
        };
      }
      return node;
    });

    const newConnections = templateData.connections.map(conn => {
      const fromId = idMap[conn.fromNodeId];
      const toId = idMap[conn.toNodeId];
      if (fromId && toId) {
        return {
          id: `conn-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`,
          fromNodeId: fromId,
          toNodeId: toId,
          controlPoint: conn.controlPoint ? { x: conn.controlPoint.x + offsetX, y: conn.controlPoint.y + offsetY } : undefined
        };
      }
      return null;
    }).filter(Boolean) as Connection[];

    setNodes(prev => [...prev, ...finalNodes]);
    setGroups(prev => [...prev, ...newGroups]);
    setConnections(prev => [...prev, ...newConnections]);
    showToastMessage(`追加了 ${finalNodes.length} 个模板卡片到当前画布`, 'success');
  };

  const handleImportJSON = (file: File) => {
    const reader = new FileReader();
    showToastMessage('正在读取并校验文件...', 'loading');
    reader.onload = (e) => {
      try {
        const data = JSON.parse(e.target?.result as string);
        
        if (!data || !Array.isArray(data.nodes)) {
          throw new Error('格式不正确：未发现 nodes 数组');
        }
        
        saveHistory();
        
        const importedNodes = data.nodes.map((n: any, idx: number) => ({
          id: n.id || `imported-node-${idx}-${Date.now()}`,
          type: n.type || 'text',
          x: typeof n.x === 'number' ? n.x : 100 + idx * 50,
          y: typeof n.y === 'number' ? n.y : 100 + idx * 50,
          width: typeof n.width === 'number' ? n.width : 320,
          height: typeof n.height === 'number' ? n.height : 250,
          title: n.title || '导入的节点',
          content: n.content || '',
          prompt: n.prompt || '',
          status: n.status || 'idle',
          model: n.model,
          ratio: n.ratio,
          resolution: n.resolution,
          duration: n.duration,
          videoMode: n.videoMode,
          count: n.count,
          mediaUrl: n.mediaUrl,
          progress: n.progress,
          isCollapsed: !!n.isCollapsed,
          groupId: n.groupId
        }));

        const importedGroups = Array.isArray(data.groups) ? data.groups.map((g: any, idx: number) => ({
          id: g.id || `imported-group-${idx}-${Date.now()}`,
          title: g.title || '导入的分组',
          color: g.color || 'cyan',
          x: typeof g.x === 'number' ? g.x : 50,
          y: typeof g.y === 'number' ? g.y : 50,
          width: typeof g.width === 'number' ? g.width : 500,
          height: typeof g.height === 'number' ? g.height : 400,
          nodeIds: Array.isArray(g.nodeIds) ? g.nodeIds : []
        })) : [];

        const importedConnections = Array.isArray(data.connections) ? data.connections.map((c: any, idx: number) => ({
          id: c.id || `imported-conn-${idx}-${Date.now()}`,
          fromNodeId: c.fromNodeId || '',
          toNodeId: c.toNodeId || '',
          controlPoint: c.controlPoint && typeof c.controlPoint.x === 'number' && typeof c.controlPoint.y === 'number' ? { x: c.controlPoint.x, y: c.controlPoint.y } : undefined
        })).filter((c: any) => c.fromNodeId && c.toNodeId) : [];

        setNodes(importedNodes);
        setGroups(importedGroups);
        setConnections(importedConnections);
        
        showToastMessage('流程导入成功！已自动自适应画布视口', 'success');
        
        setTimeout(() => {
          let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
          importedNodes.forEach((node: { x: number; y: number; width?: number; height?: number }) => {
            minX = Math.min(minX, node.x);
            minY = Math.min(minY, node.y);
            maxX = Math.max(maxX, node.x + (node.width || 320));
            maxY = Math.max(maxY, node.y + (node.height || 300));
          });
          importedGroups.forEach((group: { x: number; y: number; width?: number; height?: number }) => {
            minX = Math.min(minX, group.x);
            minY = Math.min(minY, group.y);
            maxX = Math.max(maxX, group.x + (group.width || 400));
            maxY = Math.max(maxY, group.y + (group.height || 300));
          });

          if (minX !== Infinity && containerRef.current) {
            const containerWidth = containerRef.current.clientWidth;
            const containerHeight = containerRef.current.clientHeight;
            const padding = 120;
            const contentWidth = maxX - minX;
            const contentHeight = maxY - minY;

            const scaleX = (containerWidth - padding * 2) / contentWidth;
            const scaleY = (containerHeight - padding * 2) / contentHeight;
            let newZoom = Math.min(scaleX, scaleY, 1.1);
            newZoom = Math.max(0.15, newZoom);

            const contentCenterX = minX + contentWidth / 2;
            const contentCenterY = minY + contentHeight / 2;

            setSmoothView(
              {
                x: containerWidth / 2 - contentCenterX * newZoom,
                y: containerHeight / 2 - contentCenterY * newZoom
              },
              Number(newZoom.toFixed(2))
            );
          }
        }, 100);

      } catch (err: any) {
        console.error('导入 JSON 失败', err);
        showToastMessage(`导入校验失败: ${err.message || '文件数据有误'}`, 'error', 4000);
      }
    };
    reader.onerror = () => {
      showToastMessage('读取文件发生错误', 'error');
    };
    reader.readAsText(file);
  };

  const handleStartDragControlPoint = (e: React.MouseEvent, connectionId: string) => {
    dragInitialStateRef.current = { nodes, groups, connections };
    setDraggingConnectionId(connectionId);
  };

  const handleResetControlPoint = (connectionId: string) => {
    saveHistory();
    setConnections(prev => prev.map(conn => 
      conn.id === connectionId 
        ? { ...conn, controlPoint: undefined }
        : conn
    ));
    showToastMessage('已重置连线为默认贝塞尔曲线', 'success');
  };

  const handleClearCanvas = () => {
    saveHistory();
    setNodes([]);
    setConnections([]);
    setGroups([]);
    setSelectedNodeIds([]);
    setSelectedConnectionId(null);
    setSelectedGroupId(null);
  };



  return {
    containerRef,
    nodes,
    groups,
    connections,
    toast,
    contextMenu,
    pan,
    zoom,
    isAnimatingView,
    viewportSize,
    showMinimap,
    clipboard,
    activeTool,
    selectedNodeIds,
    selectedConnectionId,
    selectedGroupId,
    draggingNodeId,
    draggingConnectionId,
    snapToGrid,
    showGrid,
    selectionBox,
    activePortDrag,
    portDragCurrentPos,
    snappingTargetNodeId,
    isStickyConnection,
    showHelp,
    showSnapshots,
    showTemplates,
    spacePressed,
    alignGuides,
    history,
    redoStack,
    setNodes,
    setContextMenu,
    setShowMinimap,
    setActiveTool,
    setSelectedNodeIds,
    setSelectedConnectionId,
    setSelectedGroupId,
    setSnapToGrid,
    setShowGrid,
    setShowHelp,
    setShowSnapshots,
    setShowTemplates,
    handleUndo,
    handleRedo,
    handleContextMenuAction,
    handleZoomIn,
    handleZoomOut,
    handleResetZoom,
    handleWheel,
    handleMouseDown,
    handleMouseMove,
    handleMouseUp,
    handleNodeDragStart,
    handlePortMouseDown,
    handleGroupDragStart,
    handleGroupResizeStart,
    handleNodeResizeStart,
    handleUpdateNode,
    handleInputPromptChange,
    handleInputSettingsChange,
    handleInputModeChange,
    handleDeleteNode,
    handleSelectNode,
    handleAddNode,
    handleCreateGroupFromSelection,
    handleBatchDelete,
    handleDisbandGroup,
    handleAlignNodes,
    handleToggleGroupCollapse,
    handleUpdateGroupTitle,
    handleUpdateGroupColor,
    handleResetView,
    handleAutoLayout,
    handleExport,
    handleExportPNG,
    handleExportPDF,
    handleRestoreSnapshot,
    handleLoadTemplate,
    handleImportJSON,
    handleStartDragControlPoint,
    handleResetControlPoint,
    handleClearCanvas,
    saveHistory,
    triggerNodeGeneration,
    showToastMessage,
    clearToast,
    setSmoothView
  };
}
