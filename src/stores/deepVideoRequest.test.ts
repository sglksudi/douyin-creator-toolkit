import {
  buildDouyinDeepAnalysisRequest,
  buildLocalVideoDeepAnalysisRequest,
  type DeepVideoAnalysisRequestPayload,
} from "./deepVideoRequest.ts";

const localRequest: DeepVideoAnalysisRequestPayload = buildLocalVideoDeepAnalysisRequest(
  {
    id: "video-1",
    path: "C:/videos/sample.mp4",
    name: "sample.mp4",
    transcript: "Limited offer. Tap now.",
  },
  "balanced",
  false
);

const douyinTextRequest: DeepVideoAnalysisRequestPayload = buildDouyinDeepAnalysisRequest(
  {
    id: "link-1",
    url: "https://v.douyin.com/example/",
    transcript: "Limited offer. Tap now.",
    videoInfo: {
      title: "Sample Douyin",
      author: "Creator",
      likes: 12,
      comments: 3,
      shares: 1,
    },
  },
  "economy",
  false
);

const douyinFrameRequest: DeepVideoAnalysisRequestPayload = buildDouyinDeepAnalysisRequest(
  {
    id: "link-2",
    url: "https://v.douyin.com/example-frame/",
    transcript: "Limited offer. Tap now.",
    localVideoPath: "C:/tmp/douyin_creator_tools/example.mp4",
  },
  "precise",
  true
);

void localRequest.source.local_video.video_path;
void douyinTextRequest.source.text_only.source_url;
void douyinFrameRequest.source.downloaded_douyin_video.video_path;

let threwMissingLocalVideoPath = false;
try {
  buildDouyinDeepAnalysisRequest(
    {
      id: "link-missing-video",
      url: "https://v.douyin.com/missing-video/",
      transcript: "Limited offer. Tap now.",
    },
    "balanced",
    true
  );
} catch (error) {
  threwMissingLocalVideoPath = error instanceof Error && error.message.includes("Frame evidence requires a cached local video");
}

if (!threwMissingLocalVideoPath) {
  throw new Error("Expected Douyin frame evidence request to require localVideoPath");
}
