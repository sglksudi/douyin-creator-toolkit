import { useVideoStore, type DeepAnalysisProfile, type VideoItem } from "./useVideoStore";

const profile: DeepAnalysisProfile = "balanced";

const analyzedVideo: VideoItem = {
  id: "video-1",
  path: "sample.mp4",
  name: "sample.mp4",
  duration_ms: 0,
  duration_str: "00:00",
  size_bytes: 0,
  size_str: "0 B",
  width: 0,
  height: 0,
  status: "completed",
  progress: 100,
  stage: "",
  transcript: "sample transcript",
  deepAnalysis: {
    status: "cancelled",
    taskId: "task-1",
  },
};

void analyzedVideo.deepAnalysis?.status;
void useVideoStore.getState().startDeepAnalysis(analyzedVideo.id, profile, false);
void useVideoStore.getState().startDeepAnalysis(analyzedVideo.id, profile, true);
