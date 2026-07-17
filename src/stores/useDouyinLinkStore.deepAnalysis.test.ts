import { useDouyinLinkStore, type LinkItem } from "./useDouyinLinkStore";

const analyzedLink: LinkItem = {
  id: "link-1",
  url: "https://v.douyin.com/example/",
  status: "success",
  retryCount: 0,
  transcript: "Limited offer. Tap now.",
  localVideoPath: "C:/tmp/douyin_creator_tools/example.mp4",
  useFrameAnalysis: false,
  deepAnalysis: {
    status: "cancelled",
    taskId: "task-1",
  },
};

void analyzedLink.deepAnalysis?.status;
void useDouyinLinkStore.getState().setUseFrameAnalysis(analyzedLink.id, true);
void useDouyinLinkStore.getState().startDeepAnalysis(analyzedLink.id, "balanced", false);
void useDouyinLinkStore.getState().startDeepAnalysis(analyzedLink.id, "balanced", true);
