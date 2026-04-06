import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface DouyinVideoInfo {
  video_url: string;
  title: string;
  author: string;
  likes: number;
  comments: number;
  shares: number;
  cover_url?: string;
  duration?: number;
}

export interface LinkParseResult {
  link: string;
  success: boolean;
  video_info?: DouyinVideoInfo;
  error?: string;
  retry_count: number;
}

export interface LinkItem {
  id: string;
  url: string;
  status: "pending" | "processing" | "success" | "failed";
  videoInfo?: DouyinVideoInfo;
  transcript?: string;
  error?: string;
  retryCount: number;
  expanded?: boolean;
}

export interface ParseProgressEvent {
  current: number;
  total: number;
  success: number;
  failed: number;
  current_link: string;
}

export interface BatchParseStats {
  total: number;
  success: number;
  failed: number;
  results: LinkParseResult[];
}

export interface McpConfig {
  dy_mcp_url: string;
  undoom_mcp_url: string;
  request_interval_ms: number;
  max_retries: number;
  timeout_secs: number;
}

interface DouyinLinkStore {
  links: LinkItem[];
  isProcessing: boolean;
  currentLinkId: string | null;
  progress: ParseProgressEvent | null;
  stats: BatchParseStats | null;

  // Service health
  dyMcpHealthy: boolean | null;
  undoomMcpHealthy: boolean | null;

  // Actions
  setLinks: (linksText: string) => void;
  addLinks: (linksText: string) => void;
  removeLink: (id: string) => void;
  clearLinks: () => void;
  toggleExpanded: (id: string) => void;

  // Processing
  parseAllLinks: () => Promise<BatchParseStats | null>;
  retryFailedLinks: () => Promise<void>;

  // Health check
  checkServicesHealth: () => Promise<void>;

  // Config
  getConfig: () => Promise<McpConfig>;
  updateConfig: (config: Partial<McpConfig>) => Promise<void>;

  // Progress listener
  setupProgressListener: () => Promise<() => void>;

  // Export
  getSuccessfulLinks: () => LinkItem[];
  getFailedLinks: () => LinkItem[];
}

let linkIdCounter = 0;

const generateLinkId = () => {
  linkIdCounter += 1;
  return `link-${Date.now()}-${linkIdCounter}`;
};

export const useDouyinLinkStore = create<DouyinLinkStore>((set, get) => ({
  links: [],
  isProcessing: false,
  currentLinkId: null,
  progress: null,
  stats: null,
  dyMcpHealthy: null,
  undoomMcpHealthy: null,

  setLinks: (linksText: string) => {
    // Regex to match Douyin URLs (v.douyin.com short links and dry standard links)
    const urlRegex = /(https?:\/\/(?:v|www)\.douyin\.com\/[^\s]+)/g;

    // Scan the entire text for URLs, ignoring surrounding text
    const matches = linksText.match(urlRegex) || [];

    // Use all matches directly (allow duplicates)
    const urls = Array.from(matches);

    const newLinks: LinkItem[] = urls.map((url) => ({
      id: generateLinkId(),
      url,
      status: "pending" as const,
      retryCount: 0,
    }));

    set({ links: newLinks, stats: null, progress: null });
  },

  addLinks: (linksText: string) => {
    // Regex to match Douyin URLs
    const urlRegex = /(https?:\/\/(?:v|www)\.douyin\.com\/[^\s]+)/g;

    const matches = linksText.match(urlRegex) || [];
    const urls = Array.from(matches);

    const newLinks: LinkItem[] = urls.map((url) => ({
      id: generateLinkId(),
      url,
      status: "pending" as const,
      retryCount: 0,
    }));

    set((state) => ({
      links: [...state.links, ...newLinks],
      stats: null,
    }));
  },

  removeLink: (id: string) => {
    set((state) => ({
      links: state.links.filter((l) => l.id !== id),
    }));
  },

  clearLinks: () => {
    set({ links: [], stats: null, progress: null });
  },

  toggleExpanded: (id: string) => {
    set((state) => ({
      links: state.links.map((l) =>
        l.id === id ? { ...l, expanded: !l.expanded } : l
      ),
    }));
  },

  parseAllLinks: async () => {
    const { links } = get();
    const pendingLinks = links.filter((l) => l.status === "pending");

    if (pendingLinks.length === 0) return null;

    set({ isProcessing: true, progress: null });

    // Mark all pending links as processing
    set((state) => ({
      links: state.links.map((l) =>
        l.status === "pending" ? { ...l, status: "processing" as const } : l
      ),
    }));

    try {
      const urls = pendingLinks.map((l) => l.url);
      const result = await invoke<BatchParseStats>("parse_douyin_links_batch", {
        links: urls,
      });

      const pendingLinkIdsByUrl = new Map<string, string[]>();
      pendingLinks.forEach((link) => {
        const ids = pendingLinkIdsByUrl.get(link.url) ?? [];
        ids.push(link.id);
        pendingLinkIdsByUrl.set(link.url, ids);
      });

      const matchedResults = result.results.map((parseResult) => ({
        parseResult,
        linkId: pendingLinkIdsByUrl.get(parseResult.link)?.shift(),
      }));

      // Update links with results
      set((state) => {
        const updatedLinks = [...state.links];

        matchedResults.forEach(({ parseResult, linkId }) => {
          if (!linkId) return;

          const linkIndex = updatedLinks.findIndex((l) => l.id === linkId);

          if (linkIndex !== -1) {
            updatedLinks[linkIndex] = {
              ...updatedLinks[linkIndex],
              status: parseResult.success ? "success" : "failed",
              videoInfo: parseResult.video_info,
              error: parseResult.error,
              retryCount: parseResult.retry_count,
              expanded: parseResult.success,
              // Init transcript as empty or undefined
            };
          }
        });

        return { links: updatedLinks, stats: result };
      });

      // [New] Automatically extract content for successful links
      const successfulLinks = matchedResults.filter(
        ({ linkId, parseResult }) => linkId && parseResult.success && parseResult.video_info
      );

      // We process them one by one to avoid overwhelming the system
      for (const { linkId, parseResult } of successfulLinks) {
        if (!linkId || !parseResult.video_info) continue;

        try {
          const transcript = await invoke<string>("extract_douyin_content", {
            url: parseResult.video_info.video_url,
            filename: parseResult.video_info.title.slice(0, 30).replace(/[\\/:*?"<>|]/g, "_") || "video",
          });

          set(state => ({
            links: state.links.map(l =>
              l.id === linkId
                ? { ...l, transcript }
                : l
            )
          }));
        } catch (e) {
          console.error("Failed to extract content:", e);
          set(state => ({
            links: state.links.map(l =>
              l.id === linkId
                ? { ...l, error: `Transcript extraction failed: ${e}` }
                : l
            )
          }));
        }
      }

      return result;
    } catch (error) {
      // Mark all processing links as failed
      set((state) => ({
        links: state.links.map((l) =>
          l.status === "processing"
            ? { ...l, status: "failed" as const, error: String(error) }
            : l
        ),
      }));
      throw error;
    } finally {
      set({ isProcessing: false });
    }
  },

  retryFailedLinks: async () => {
    // Reset failed links to pending
    set((state) => ({
      links: state.links.map((l) =>
        l.status === "failed"
          ? { ...l, status: "pending" as const, error: undefined }
          : l
      ),
    }));

    // Re-parse
    await get().parseAllLinks();
  },

  checkServicesHealth: async () => {
    try {
      const [dyHealth, undoomHealth] = await Promise.all([
        invoke<boolean>("check_dy_mcp_health"),
        invoke<boolean>("check_undoom_mcp_health"),
      ]);

      set({
        dyMcpHealthy: dyHealth,
        undoomMcpHealthy: undoomHealth,
      });
    } catch (error) {
      set({
        dyMcpHealthy: false,
        undoomMcpHealthy: false,
      });
    }
  },

  getConfig: async () => {
    return await invoke<McpConfig>("get_mcp_config");
  },

  updateConfig: async (config: Partial<McpConfig>) => {
    await invoke("update_mcp_config", config);
  },

  setupProgressListener: async () => {
    const unlisten = await listen<ParseProgressEvent>(
      "mcp:parse-progress",
      (event) => {
        const progress = event.payload;
        set({ progress });

        // Update current link status
        set((state) => ({
          links: state.links.map((l) =>
            l.url === progress.current_link
              ? { ...l, status: "processing" as const }
              : l
          ),
        }));
      }
    );

    return unlisten;
  },

  getSuccessfulLinks: () => {
    return get().links.filter((l) => l.status === "success");
  },

  getFailedLinks: () => {
    return get().links.filter((l) => l.status === "failed");
  },
}));
