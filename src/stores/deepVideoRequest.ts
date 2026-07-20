export type DeepAnalysisProfile = "economy" | "balanced" | "precise";

export interface TranscriptPayload {
  text: string;
  segments: Array<{
    text: string;
    start_seconds?: number | null;
    end_seconds?: number | null;
  }>;
}

interface DeepVideoRequestBase {
  task_id: string;
  title: string;
  profile: DeepAnalysisProfile;
  use_frame_analysis: boolean;
  transcript: TranscriptPayload | null;
  ocr_items: [];
  reference_text: string | null;
}

export interface LocalVideoDeepAnalysisRequestPayload extends DeepVideoRequestBase {
  source: {
    local_video: {
      video_path: string;
    };
  };
}

export interface DouyinTextOnlyDeepAnalysisRequestPayload extends DeepVideoRequestBase {
  source: {
    text_only: {
      source_url: string;
    };
  };
  transcript: TranscriptPayload;
}

export interface DouyinFrameDeepAnalysisRequestPayload extends DeepVideoRequestBase {
  source: {
    downloaded_douyin_video: {
      video_path: string;
      source_url: string;
    };
  };
  transcript: TranscriptPayload;
}

export type DeepVideoAnalysisRequestPayload =
  | LocalVideoDeepAnalysisRequestPayload
  | DouyinTextOnlyDeepAnalysisRequestPayload
  | DouyinFrameDeepAnalysisRequestPayload;

export interface LocalVideoDeepAnalysisInput {
  id: string;
  path: string;
  name: string;
  transcript?: string;
}

export interface DouyinDeepAnalysisInput {
  id: string;
  url: string;
  transcript: string;
  localVideoPath?: string;
  videoInfo?: {
    title: string;
    author: string;
    likes: number;
    comments: number;
    shares: number;
  };
}

function transcriptPayload(text: string | undefined): TranscriptPayload | null {
  return text ? { text, segments: [] } : null;
}

function douyinReferenceText(input: DouyinDeepAnalysisInput): string | null {
  return input.videoInfo
    ? `Author: ${input.videoInfo.author}\nLikes: ${input.videoInfo.likes}\nComments: ${input.videoInfo.comments}\nShares: ${input.videoInfo.shares}`
    : null;
}

export function buildLocalVideoDeepAnalysisRequest(
  video: LocalVideoDeepAnalysisInput,
  profile: DeepAnalysisProfile,
  useFrameAnalysis: boolean
): LocalVideoDeepAnalysisRequestPayload {
  return {
    source: { local_video: { video_path: video.path } },
    task_id: video.id,
    title: video.name,
    profile,
    use_frame_analysis: useFrameAnalysis,
    transcript: transcriptPayload(video.transcript),
    ocr_items: [],
    reference_text: null,
  };
}

export function buildDouyinDeepAnalysisRequest(
  link: DouyinDeepAnalysisInput,
  profile: DeepAnalysisProfile,
  useFrameAnalysis: false
): DouyinTextOnlyDeepAnalysisRequestPayload;
export function buildDouyinDeepAnalysisRequest(
  link: DouyinDeepAnalysisInput,
  profile: DeepAnalysisProfile,
  useFrameAnalysis: true
): DouyinFrameDeepAnalysisRequestPayload;
export function buildDouyinDeepAnalysisRequest(
  link: DouyinDeepAnalysisInput,
  profile: DeepAnalysisProfile,
  useFrameAnalysis: boolean
): DouyinTextOnlyDeepAnalysisRequestPayload | DouyinFrameDeepAnalysisRequestPayload;
export function buildDouyinDeepAnalysisRequest(
  link: DouyinDeepAnalysisInput,
  profile: DeepAnalysisProfile,
  useFrameAnalysis: boolean
): DouyinTextOnlyDeepAnalysisRequestPayload | DouyinFrameDeepAnalysisRequestPayload {
  const base: Omit<DouyinTextOnlyDeepAnalysisRequestPayload, "source"> = {
    task_id: link.id,
    title: link.videoInfo?.title || link.url,
    profile,
    use_frame_analysis: useFrameAnalysis,
    transcript: { text: link.transcript, segments: [] },
    ocr_items: [],
    reference_text: douyinReferenceText(link),
  };

  if (useFrameAnalysis) {
    if (!link.localVideoPath) {
      throw new Error("Frame evidence requires a cached local video. Re-extract the link transcript first.");
    }

    return {
      ...base,
      source: {
        downloaded_douyin_video: {
          video_path: link.localVideoPath,
          source_url: link.url,
        },
      },
    };
  }

  return {
    ...base,
    source: {
      text_only: {
        source_url: link.url,
      },
    },
  };
}
