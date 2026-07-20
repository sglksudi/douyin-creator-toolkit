import { readFileSync } from "node:fs";

const files = {
  app: "src/App.tsx",
  sidebar: "src/components/layout/Sidebar.tsx",
  localVideo: "src/pages/LocalVideo.tsx",
  douyinLink: "src/pages/DouyinLink.tsx",
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

const app = read(files.app);
const sidebar = read(files.sidebar);
const localVideo = read(files.localVideo);
const douyinLink = read(files.douyinLink);

[
  "local-video",
  "douyin-link",
  "video-download",
  "agent-studio",
  "knowledge-base",
  "tasks",
  "about",
].forEach((route) => assertRoute(app, sidebar, route));

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
