import { readFileSync } from "node:fs";

const files = {
  app: "src/App.tsx",
  sidebar: "src/components/layout/Sidebar.tsx",
  localVideo: "src/pages/LocalVideo.tsx",
  douyinLink: "src/pages/DouyinLink.tsx",
  settings: "src/pages/Settings.tsx",
  taskHistory: "src/pages/TaskHistory.tsx",
};

function read(path) {
  return readFileSync(path, "utf8");
}

function assertContains(content, needle, label) {
  if (!content.includes(needle)) {
    throw new Error(`${label}: expected to find ${JSON.stringify(needle)}`);
  }
}

function assertRoute(app, sidebar, route) {
  assertContains(app, `case "${route}"`, `App route ${route}`);
  assertContains(sidebar, `id: "${route}"`, `Sidebar nav ${route}`);
}

function assertCriticalNavigationPage({ route, render, navigation, content, page }) {
  assertContains(app, `case "${route}"`, `App route ${route}`);
  assertContains(app, render, `App render ${route}`);
  assertContains(sidebar, navigation, `Sidebar navigation ${route}`);
  content.forEach((needle) => assertContains(page, needle, `Page content ${route}`));
}

const app = read(files.app);
const sidebar = read(files.sidebar);
const localVideo = read(files.localVideo);
const douyinLink = read(files.douyinLink);
const settings = read(files.settings);
const taskHistory = read(files.taskHistory);

[
  "local-video",
  "douyin-link",
  "video-download",
  "agent-studio",
  "knowledge-base",
  "tasks",
  "about",
].forEach((route) => assertRoute(app, sidebar, route));

const criticalNavigationPages = [
  {
    route: "local-video",
    render: "return <LocalVideo />",
    navigation: "id: \"local-video\"",
    page: localVideo,
    content: ["本地视频", "handleDeepAnalyze", "证据链分析"],
  },
  {
    route: "douyin-link",
    render: "return <DouyinLink />",
    navigation: "id: \"douyin-link\"",
    page: douyinLink,
    content: ["抖音链接文案提取", "开始解析", "startDeepAnalysis"],
  },
  {
    route: "tasks",
    render: "return <TaskHistory />",
    navigation: "id: \"tasks\"",
    page: taskHistory,
    content: ["任务队列", "handleRefresh", "历史记录"],
  },
  {
    route: "settings",
    render: "return <Settings />",
    navigation: "onTabChange(\"settings\")",
    page: settings,
    content: ["保存设置", "AI 设置状态", "网络设置"],
  },
];

criticalNavigationPages.forEach(assertCriticalNavigationPage);

assertContains(app, "register(useVideoStore.getState().setupProgressListener)", "App video listener");
assertContains(app, "register(useDouyinLinkStore.getState().setupProgressListener)", "App Douyin listener");
assertContains(app, "setActiveTab(\"tasks\")", "Tray task navigation");
assertContains(app, "case \"settings\"", "App Settings route");
assertContains(app, "default:", "App default route fallback");
assertContains(app, "return <LocalVideo />", "App default LocalVideo fallback");
assertContains(app, "return <Settings />", "App Settings route render");
assertContains(sidebar, "onTabChange(\"settings\")", "Sidebar Settings footer navigation");

assertContains(localVideo, "setUseFrameAnalysis(video.id, enabled)", "LocalVideo frame evidence switch");
assertContains(localVideo, "startDeepAnalysis(video.id, deepProfile, Boolean(video.useFrameAnalysis))", "LocalVideo deep analysis action");
assertContains(localVideo, "aria-label=\"", "LocalVideo accessible switch label");
assertContains(localVideo, "disabled={!video.useFrameAnalysis}", "LocalVideo profile disabled in text mode");

assertContains(douyinLink, "setUseFrameAnalysis(link.id, checked)", "DouyinLink frame evidence switch");
assertContains(douyinLink, "startDeepAnalysis(link.id, \"balanced\", Boolean(link.useFrameAnalysis))", "DouyinLink deep analysis action");
assertContains(douyinLink, "description: link.useFrameAnalysis", "DouyinLink mode-specific toast");
assertContains(douyinLink, "disabled={!link.transcript || link.deepAnalysis?.status === \"running\"}", "DouyinLink analysis button guard");

console.log("UI smoke checks passed");
