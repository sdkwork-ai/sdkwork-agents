import type {
  GenerationRecord,
} from '@sdkwork/agents-pc-core/sdk/generationsService';

import type { CanvasGroup, CanvasNode, Connection } from '../types';

async function loadGenerationsService() {
  const { agentsGenerationsService } = await import(
    '@sdkwork/agents-pc-core/sdk/generationsService'
  );
  return agentsGenerationsService;
}

const INITIAL_NODES: CanvasNode[] = [
  {
    id: 'node-1',
    type: 'text',
    x: 140,
    y: 160,
    width: 320,
    height: 250,
    title: '第1步：剧本构想',
    content: '在一个充满赛博朋克霓虹的未来都市中，一个小型的发光Octo章鱼侦探，正在雨夜中寻找神秘的代码卷轴...',
    groupId: 'group-initial'
  },
  {
    id: 'node-2',
    type: 'image-gen',
    x: 550,
    y: 120,
    width: 320,
    height: 380,
    title: '第2步：角色渲染',
    prompt: '在一个充满赛博朋克霓虹的未来都市中，一个小型的发光Octo章鱼侦探，正在雨夜中寻找神秘的代码卷轴...',
    model: '5.0-lite',
    ratio: '16:9',
    status: 'idle',
    groupId: 'group-initial'
  },
  {
    id: 'node-3',
    type: 'video-gen',
    x: 1000,
    y: 180,
    width: 320,
    height: 340,
    title: '第3步：动态镜头',
    model: '2.0-fast',
    duration: 5,
    status: 'idle',
    refImageNodeId: 'node-2'
  }
];

const INITIAL_GROUPS: CanvasGroup[] = [
  {
    id: 'group-initial',
    title: '剧本与创意大纲阶段',
    color: 'cyan',
    x: 100,
    y: 60,
    width: 820,
    height: 480,
    nodeIds: ['node-1', 'node-2']
  }
];

const INITIAL_CONNECTIONS: Connection[] = [
  { id: 'conn-1', fromNodeId: 'node-1', toNodeId: 'node-2' },
  { id: 'conn-2', fromNodeId: 'node-2', toNodeId: 'node-3' }
];

const LOCAL_STORAGE_KEY = 'sdkwork_agents_canvas_workflow_v1';

export class CanvasService {
  /**
   * Fetch initial workflow state.
   *
   * NOTE: the canvas workflow graph is intentionally browser-local today.
   * Media generation (image/video) already calls the real generations API;
   * server-side persistence of the workflow graph itself is a product
   * decision pending a backend resource. No server state is implied.
   */
  static async getInitialWorkflow() {
    try {
      const saved = localStorage.getItem(LOCAL_STORAGE_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        if (parsed && Array.isArray(parsed.nodes)) {
          parsed.nodes = parsed.nodes.map((node: any) => {
            if (node.type === 'image-gen' || node.type === 'video-gen') {
              let updated = false;
              let w = node.width;
              let h = node.height;
              // Reset/Clamp huge dimensions caused by the settings conflict bug
              if (typeof node.width === 'number' && node.width > 360) {
                w = 260;
                updated = true;
              }
              // If it was corrupt/huge, recalculate height using the ratio
              if (updated || (typeof node.height === 'number' && node.height > 550)) {
                let numericRatio = 1.0;
                const ratioStr = node.ratio || '1:1';
                const parts = ratioStr.split(':');
                if (parts.length === 2) {
                  const rW = parseFloat(parts[0]);
                  const rH = parseFloat(parts[1]);
                  if (rW > 0 && rH > 0) {
                    numericRatio = rW / rH;
                  }
                }
                h = Math.round(w / numericRatio) + 37;
                updated = true;
              }
              if (updated) {
                return { ...node, width: w, height: h };
              }
            }
            return node;
          });
        }
        return parsed;
      }
    } catch (e) {
      console.warn('Failed to parse saved workflow', e);
    }
    return {
      nodes: [...INITIAL_NODES],
      groups: [...INITIAL_GROUPS],
      connections: [...INITIAL_CONNECTIONS]
    };
  }

  static saveWorkflow(data: { nodes: CanvasNode[], groups: CanvasGroup[], connections: Connection[] }) {
    try {
      localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(data));
    } catch (e) {
      console.warn('Failed to save workflow', e);
    }
  }

  private static toProgress(record: GenerationRecord): number {
    if (record.status === 'succeeded') return 100;
    if (record.status === 'running') return 67;
    return 29;
  }

  private static toProgressMessage(record: GenerationRecord): string {
    if (record.status === 'succeeded') return '生成完毕';
    if (record.status === 'running') return '正在渲染生成结果...';
    return '生成任务已进入队列...';
  }

  static async generateImage(prompt: string, ratio: string, onProgress: (p: number, msg: string) => void): Promise<string> {
    const generationsService = await loadGenerationsService();
    const command = await generationsService.create({
      modality: 'image',
      operationType: 'text_to_image',
      prompt,
      parameters: { aspectRatio: ratio },
    });
    const record = await generationsService.waitForCompletion(command.generation, {
      onStatus(current) {
        onProgress(
          CanvasService.toProgress(current),
          CanvasService.toProgressMessage(current),
        );
      },
    });
    const media = await generationsService.listMediaResults(record.id);
    const image = media.find(item => item.kind === 'image');
    if (!image) {
      throw new Error('图片生成完成，但未返回可预览的图片资源。');
    }
    return image.url;
  }

  static async generateVideo(prompt: string, onProgress: (p: number, msg: string) => void): Promise<string> {
    const generationsService = await loadGenerationsService();
    const command = await generationsService.create({
      modality: 'video',
      operationType: 'text_to_video',
      prompt,
    });
    const record = await generationsService.waitForCompletion(command.generation, {
      onStatus(current) {
        onProgress(
          CanvasService.toProgress(current),
          CanvasService.toProgressMessage(current),
        );
      },
    });
    const media = await generationsService.listMediaResults(record.id);
    const video = media.find(item => item.kind === 'video');
    if (!video) {
      throw new Error('视频生成完成，但未返回可预览的视频资源。');
    }
    return video.url;
  }
}
