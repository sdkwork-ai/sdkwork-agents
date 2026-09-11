import { useState, useRef, useEffect } from 'react';
import { CanvasNode } from '../types';
import { CanvasService } from '../services/CanvasService';

export function useFlowCard(
  node: CanvasNode,
  onUpdate: (id: string, updates: Partial<CanvasNode>) => void,
  onSelect: (id: string, e: React.MouseEvent) => void,
  onDragStart: (id: string, e: React.MouseEvent) => void,
  connectedInputNode?: CanvasNode
) {
  const [isPlaying, setIsPlaying] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);
  const [isHovered, setIsHovered] = useState(false);

  const [activeDropdown, setActiveDropdown] = useState<'model' | 'ratio' | 'duration' | 'count' | 'videoMode' | 'resolution' | null>(null);
  const cardDropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (activeDropdown && cardDropdownRef.current && !cardDropdownRef.current.contains(e.target as Node)) {
        setActiveDropdown(null);
      }
    };
    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  }, [activeDropdown]);

  // Sync content or prompts based on connections
  useEffect(() => {
    if (node.type === 'image-gen' && connectedInputNode) {
      // Auto inherit prompt from text node
      if (connectedInputNode.type === 'text') {
        const inheritedText = connectedInputNode.content || '';
        // If local prompt is empty or we force sync
        if (node.prompt !== inheritedText && inheritedText.trim().length > 0) {
          onUpdate(node.id, { 
            prompt: inheritedText, 
            refTextNodeId: connectedInputNode.id 
          });
        }
      }
    }
    if (node.type === 'video-gen' && connectedInputNode) {
      if (connectedInputNode.type === 'image-gen') {
        if (node.refImageNodeId !== connectedInputNode.id) {
          onUpdate(node.id, { 
            refImageNodeId: connectedInputNode.id,
            prompt: node.prompt || connectedInputNode.prompt 
          });
        }
      } else if (connectedInputNode.type === 'text') {
        if (node.prompt !== connectedInputNode.content) {
          onUpdate(node.id, { 
            prompt: connectedInputNode.content,
            refTextNodeId: connectedInputNode.id
          });
        }
      }
    }
  }, [connectedInputNode?.id, connectedInputNode?.content, connectedInputNode?.content]);

  // Handle Dragging Check
  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return; // Left button only
    const target = e.target as HTMLElement;
    
    // Stop propagation so clicking inside the card does not deselect it or start background lasso selection
    e.stopPropagation();

    // Call onSelect on every left-mouse down inside the card so the card is selected!
    onSelect(node.id, e);

    if (
      target.closest('input') || 
      target.closest('textarea') || 
      target.closest('button') || 
      target.closest('select') ||
      target.closest('.port-dot') ||
      target.closest('.no-drag')
    ) {
      // Return early and do not trigger dragging, allowing normal interactive focus and click states
      return;
    }
    e.preventDefault();
    onDragStart(node.id, e);
  };

  // Run mock generation steps
  const triggerGeneration = () => {
    if (node.status === 'generating') return;

    onUpdate(node.id, { status: 'generating', progress: 5, mediaUrl: undefined });

    if (node.type === 'text') {
      setTimeout(() => {
        onUpdate(node.id, { 
          status: 'idle', 
          progress: 100, 
          content: `${node.content || ''}\n\n[AI]: 这里是根据提示词 "${node.prompt}" 生成的文本内容。`.trim()
        });
      }, 800);
    } else if (node.type === 'image-gen') {
      CanvasService.generateImage(node.prompt || '', node.ratio || '1:1', (p, msg) => {
        onUpdate(node.id, { progress: p, content: msg });
      }).then((url) => {
        onUpdate(node.id, { status: 'completed', progress: 100, mediaUrl: url });
      });
    } else if (node.type === 'video-gen') {
      CanvasService.generateVideo(node.prompt || '', (p, msg) => {
        onUpdate(node.id, { progress: p, content: msg });
      }).then((url) => {
        onUpdate(node.id, { status: 'completed', progress: 100, mediaUrl: url });
      });
    }
  };

  const handleVideoPlayToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!videoRef.current) return;
    if (isPlaying) {
      videoRef.current.pause();
      setIsPlaying(false);
    } else {
      videoRef.current.play().then(() => {
        setIsPlaying(true);
      }).catch(err => console.log(err));
    }
  };

  const cardRef = useRef<HTMLDivElement>(null);

  // Sync actual rendered height back to node state for perfect connection lines
  useEffect(() => {
    if (!cardRef.current) return;
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const height = entry.contentRect.height;
        // Check if changed significantly to avoid infinite loops and micro-adjustments
        if (node.height === undefined || Math.abs(node.height - height) > 2) {
          onUpdate(node.id, { height: Math.round(height) });
        }
      }
    });
    observer.observe(cardRef.current);
    return () => observer.disconnect();
  }, [node.id, node.height, onUpdate]);


  return {
    isPlaying,
    setIsPlaying,
    videoRef,
    isHovered,
    setIsHovered,
    activeDropdown,
    setActiveDropdown,
    cardDropdownRef,
    handleMouseDown,
    handleVideoPlayToggle,
    triggerGeneration,
    cardRef
  };
}
