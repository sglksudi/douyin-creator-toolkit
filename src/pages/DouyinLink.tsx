import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/input";
import {
  Link,
  Play,
  FileText,
  CheckCircle2,
  XCircle,
  Loader2,
  RefreshCw,
  Trash2,
  ChevronDown,
  ChevronRight,
  AlertCircle,
  Heart,
  MessageCircle,
  Share2,
  User,
  Copy,
  ExternalLink,
  Brain,
  Layers,
} from "lucide-react";
import { useDouyinLinkStore, LinkItem } from "@/stores/useDouyinLinkStore";
import { useToast } from "@/hooks/useToast";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "@/stores/useAppStore";
import { AIChatModal } from "@/components/AIChatModal";

function LinkResultCard({
  link,
  onChat,
  onSendToAgent,
}: {
  link: LinkItem;
  onChat: (link: LinkItem) => void;
  onSendToAgent: (link: LinkItem) => void;
}) {
  const { toggleExpanded, removeLink, setUseFrameAnalysis, startDeepAnalysis } = useDouyinLinkStore();
  const { toast } = useToast();

  const handleDeepAnalysis = async () => {
    if (!link.transcript) {
      toast({
        title: "无法分析",
        description: "请先等待视频文案提取完成",
        variant: "warning",
      });
      return;
    }

    try {
      await startDeepAnalysis(link.id, "balanced", Boolean(link.useFrameAnalysis));
      toast({
        title: "证据链分析已启动",
        description: link.useFrameAnalysis
          ? "画面证据已开启，将复用缓存视频抽取画面"
          : "文案模式已启动，不会额外下载或抽帧",
      });
    } catch (error) {
      toast({
        title: "证据链分析失败",
        description: String(error),
        variant: "error",
      });
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    toast({
      title: "已复制",
      description: "链接已复制到剪贴板",
    });
  };

  const openInBrowser = (url: string) => {
    window.open(url, "_blank");
  };

  return (
    <div
      className={`group rounded-xl bg-white dark:bg-zinc-900/50 border transition-all duration-200 ${link.status === "success"
        ? "border-emerald-200 dark:border-emerald-900/50"
        : link.status === "failed"
          ? "border-red-200 dark:border-red-900/50"
          : "border-zinc-200/80 dark:border-zinc-800/80"
        }`}
    >
      {/* Header */}
      <div
        className="flex items-center gap-3 p-4 cursor-pointer"
        onClick={() => link.status === "success" && toggleExpanded(link.id)}
      >
        {/* Status Icon */}
        <div className="flex-shrink-0">
          {link.status === "success" && (
            <CheckCircle2 className="w-5 h-5 text-emerald-500" />
          )}
          {link.status === "failed" && (
            <XCircle className="w-5 h-5 text-red-500" />
          )}
          {link.status === "processing" && (
            <Loader2 className="w-5 h-5 text-[#1976D2] animate-spin" />
          )}
          {link.status === "pending" && (
            <div className="w-5 h-5 rounded-full border-2 border-zinc-300 dark:border-zinc-600" />
          )}
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <p className="text-[13px] text-zinc-500 dark:text-zinc-400 truncate font-mono">
            {link.url}
          </p>
          {link.videoInfo && (
            <p className="text-[14px] text-zinc-800 dark:text-zinc-100 mt-0.5 truncate">
              {link.videoInfo.title}
            </p>
          )}
          {link.error && (
            <p className="text-[13px] text-red-500 mt-0.5 flex items-center gap-1">
              <AlertCircle className="w-3.5 h-3.5" />
              {link.error}
              {link.retryCount > 0 && (
                <span className="text-zinc-400">
                  （已重试 {link.retryCount} 次）
                </span>
              )}
            </p>
          )}
        </div>

        {/* Actions */}
        <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
          <button
            onClick={(e) => {
              e.stopPropagation();
              copyToClipboard(link.url);
            }}
            className="p-1.5 rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300"
            title="复制链接"
          >
            <Copy className="w-4 h-4" />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              openInBrowser(link.url);
            }}
            className="p-1.5 rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300"
            title="在浏览器中打开"
          >
            <ExternalLink className="w-4 h-4" />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              removeLink(link.id);
            }}
            className="p-1.5 rounded-md hover:bg-red-50 dark:hover:bg-red-900/20 text-zinc-400 hover:text-red-500"
            title="删除"
          >
            <Trash2 className="w-4 h-4" />
          </button>
        </div>

        {/* Expand Icon */}
        {link.status === "success" && (
          <div className="flex-shrink-0 text-zinc-400">
            {link.expanded ? (
              <ChevronDown className="w-5 h-5" />
            ) : (
              <ChevronRight className="w-5 h-5" />
            )}
          </div>
        )}
      </div>

      {/* Expanded Content */}
      {link.expanded && link.videoInfo && (
        <div className="px-4 pb-4 pt-0 border-t border-zinc-100 dark:border-zinc-800">
          <div className="mt-3 space-y-3">
            {/* Author */}
            <div className="flex items-center gap-2 text-[13px] text-zinc-500 dark:text-zinc-400">
              <User className="w-4 h-4" />
              <span>{link.videoInfo.author}</span>
            </div>

            {/* Stats */}
            <div className="flex items-center gap-4 text-[13px] text-zinc-500 dark:text-zinc-400">
              <div className="flex items-center gap-1">
                <Heart className="w-4 h-4 text-red-400" />
                <span>{formatNumber(link.videoInfo.likes)}</span>
              </div>
              <div className="flex items-center gap-1">
                <MessageCircle className="w-4 h-4 text-blue-400" />
                <span>{formatNumber(link.videoInfo.comments)}</span>
              </div>
              <div className="flex items-center gap-1">
                <Share2 className="w-4 h-4 text-green-400" />
                <span>{formatNumber(link.videoInfo.shares)}</span>
              </div>
            </div>

            {/* Video URL */}
            {link.videoInfo.video_url && (
              <div className="flex items-center gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  className="rounded-lg text-[13px]"
                  onClick={() => copyToClipboard(link.videoInfo!.video_url)}
                >
                  <Copy className="w-3.5 h-3.5 mr-1.5" />
                  复制无水印链接
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  className="rounded-lg text-[13px]"
                  onClick={() => openInBrowser(link.videoInfo!.video_url)}
                >
                  <ExternalLink className="w-3.5 h-3.5 mr-1.5" />
                  打开视频
                </Button>
              </div>
            )}

            <div className="mt-4 p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg border border-zinc-100 dark:border-zinc-800">
              <div className="flex items-center justify-between mb-2">
                <h4 className="text-[13px] font-medium text-zinc-700 dark:text-zinc-300">视频文案</h4>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="ghost"
                    className="h-6 text-[12px] text-zinc-400 hover:text-zinc-600"
                    onClick={() => copyToClipboard(link.transcript!)}
                  >
                    <Copy className="w-3 h-3 mr-1" />
                    复制
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    className="h-6 text-[12px] text-indigo-500 hover:text-indigo-600 hover:bg-indigo-50 dark:hover:bg-indigo-900/10"
                    onClick={() => onChat(link)}
                  >
                    <MessageCircle className="w-3 h-3 mr-1" />
                    AI 对话
                  </Button>
                  <div className="flex h-6 items-center gap-1.5 rounded-md border border-teal-200 bg-white px-2 text-[12px] text-teal-700 dark:border-teal-800 dark:bg-zinc-900 dark:text-teal-300">
                    <span>{link.useFrameAnalysis ? "画面证据" : "文案模式"}</span>
                    <Switch
                      checked={Boolean(link.useFrameAnalysis)}
                      onCheckedChange={(checked) => setUseFrameAnalysis(link.id, checked)}
                      aria-label="画面证据"
                    />
                  </div>
                  <Button
                    size="sm"
                    variant="ghost"
                    className="h-6 text-[12px] text-teal-600 hover:text-teal-700 hover:bg-teal-50 dark:hover:bg-teal-900/10"
                    onClick={handleDeepAnalysis}
                    disabled={!link.transcript || link.deepAnalysis?.status === "running"}
                  >
                    <Layers className="w-3 h-3 mr-1" />
                    证据链分析
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    className="h-6 text-[12px] text-blue-500 hover:text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/10"
                    onClick={() => onSendToAgent(link)}
                  >
                    <Brain className="w-3 h-3 mr-1" />
                    发送到 Agent
                  </Button>
                </div>
              </div>
              {link.transcript ? (
                <>
                  <p className="text-[13px] text-zinc-600 dark:text-zinc-400 leading-relaxed whitespace-pre-wrap">
                    {link.transcript}
                  </p>
                  {link.deepAnalysis && (
                    <div className="mt-3 rounded-lg border border-teal-200/80 bg-teal-50/70 p-3 text-xs text-teal-800 shadow-sm dark:border-teal-900/80 dark:bg-teal-950/20 dark:text-teal-200">
                      <div className="flex flex-wrap items-center gap-2">
                        <Layers className="w-4 h-4 text-teal-600 dark:text-teal-300" />
                        <span className="font-medium">证据链分析</span>
                        <span className="rounded-full bg-white/80 px-2 py-0.5 text-[11px] text-teal-700 dark:bg-teal-900/50 dark:text-teal-200">
                          {link.deepAnalysis.status === "running"
                            ? link.deepAnalysis.progress
                              ? `分析中 ${link.deepAnalysis.progress}%`
                              : "分析中"
                            : link.deepAnalysis.status === "completed"
                              ? "已完成"
                              : link.deepAnalysis.status === "cancelled"
                                ? "\u5df2\u53d6\u6d88"
                                : link.deepAnalysis.status === "failed"
                                ? "失败"
                                : "未开始"}
                        </span>
                        <span className="rounded-full bg-white/80 px-2 py-0.5 text-[11px] text-teal-700 dark:bg-teal-900/50 dark:text-teal-200">
                          {link.deepAnalysis.useFrameAnalysis ? "画面证据" : "文案模式"}
                        </span>
                      </div>
                      {link.deepAnalysis.status === "running" && link.deepAnalysis.progress !== undefined && (
                        <div className="mt-3 h-1.5 w-full overflow-hidden rounded-full bg-teal-100 dark:bg-teal-950">
                          <div
                            className="h-full rounded-full bg-teal-500 transition-all duration-300"
                            style={{ width: `${link.deepAnalysis.progress}%` }}
                          />
                        </div>
                      )}
                      {link.deepAnalysis.resultPath && (
                        <div className="mt-2 break-all text-teal-700/80 dark:text-teal-200/80">
                          结果：{link.deepAnalysis.resultPath}
                        </div>
                      )}
                      {link.deepAnalysis.error && (
                        <div className="mt-2 break-all text-red-600 dark:text-red-400">
                          {link.deepAnalysis.error}
                        </div>
                      )}
                    </div>
                  )}
                </>
              ) : (
                <div className="mt-2 flex items-center gap-2 text-[13px] text-zinc-400">
                  <Loader2 className="w-3 h-3 animate-spin" />
                  <span>正在提取文案...</span>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function formatNumber(num: number): string {
  if (num >= 10000) {
    return (num / 10000).toFixed(1) + "万";
  }
  return num.toString();
}

export function DouyinLink() {
  const [linksText, setLinksText] = useState("");
  const { settings, setActiveTab, setAgentDraft } = useAppStore();
  const {
    links,
    isProcessing,
    progress,
    stats,
    dyMcpHealthy,
    setLinks,
    clearLinks,
    parseAllLinks,
    retryFailedLinks,
    checkServicesHealth,
    getSuccessfulLinks,
    getFailedLinks,
  } = useDouyinLinkStore();
  const { toast } = useToast();

  // Chat Modal State
  const [chatOpen, setChatOpen] = useState(false);
  const [chatContext, setChatContext] = useState("");
  const [chatTitle, setChatTitle] = useState("");

  const handleChat = (link: LinkItem) => {
    setChatContext(link.transcript || "");
    setChatTitle(link.videoInfo?.title || "视频文案");
    setChatOpen(true);
  };

  const linkCount = (linksText.match(/(https?:\/\/(?:v|www)\.douyin\.com\/[^\s]+)/g) || []).length;
  const successCount = getSuccessfulLinks().length;
  const failedCount = getFailedLinks().length;

  // Check services health on mount
  useEffect(() => {
    checkServicesHealth();
  }, [checkServicesHealth]);

  const handleStartParse = async () => {
    if (linkCount === 0) return;

    // Set links from text
    setLinks(linksText);

    try {
      const result = await parseAllLinks();
      if (result) {
        toast({
          title: "解析完成",
          description: `成功 ${result.success} 个，失败 ${result.failed} 个`,
        });
      }
    } catch (error) {
      toast({
        title: "解析失败",
        description: String(error),
        variant: "error",
      });
    }
  };

  const handleRetry = async () => {
    try {
      await retryFailedLinks();
      toast({
        title: "重试完成",
        description: "已重新解析失败的链接",
      });
    } catch (error) {
      toast({
        title: "重试失败",
        description: String(error),
        variant: "error",
      });
    }
  };

  const handleSendToAgent = (link: LinkItem) => {
    if (!link.transcript || !link.videoInfo) {
      toast({
        title: "无法发送",
        description: "请先等待链接解析和文案提取完成",
        variant: "warning",
      });
      return;
    }

    const referenceText = [
      `标题：${link.videoInfo.title}`,
      `作者：${link.videoInfo.author}`,
      `点赞：${formatNumber(link.videoInfo.likes)} / 评论：${formatNumber(link.videoInfo.comments)} / 分享：${formatNumber(link.videoInfo.shares)}`,
      `原始链接：${link.url}`,
    ].join("\n");

    setAgentDraft({
      task: "请分析这条抖音内容为什么可能有效，提炼 3 个可复用的爆点钩子，并改写成一版更适合我账号发布的口播脚本。",
      transcript: link.transcript,
      referenceText,
      preferredSkillId: "douyin-hot",
      useKnowledgeBase: true,
    });
    setActiveTab("agent-studio");
    toast({
      title: "已发送到 Agent",
      description: "你可以在 Agent Studio 继续做热点分析和脚本改写",
    });
  };

  const handleExportDoc = async () => {
    const successfulLinks = getSuccessfulLinks();
    if (successfulLinks.length === 0) {
      toast({
        title: "没有可导出的内容",
        description: "请先解析视频链接",
        variant: "error",
      });
      return;
    }

    if (!settings.defaultExportPath) {
      toast({
        title: "未设置导出路径",
        description: "请在设置页配置默认导出路径",
        variant: "error",
      });
      return;
    }

    try {
      const date = new Date();
      const timestamp = date.getFullYear().toString() +
        (date.getMonth() + 1).toString().padStart(2, '0') +
        date.getDate().toString().padStart(2, '0') + "_" +
        date.getHours().toString().padStart(2, '0') +
        date.getMinutes().toString().padStart(2, '0');

      const filename = `抖音文案提取_${timestamp}.docx`;
      const basePath = settings.defaultExportPath.replace(/[\\/]$/, "");
      const fullPath = `${basePath}\\${filename}`;

      const videos = successfulLinks.map(link => ({
        video_id: link.id,
        video_name: link.videoInfo?.title || link.url,
        transcript: link.transcript || "",
        error: null,
        duration_ms: (link.videoInfo?.duration || 0)
      }));

      await invoke("export_transcripts_to_docx", {
        videos,
        outputPath: fullPath
      });

      toast({
        title: "导出成功",
        description: `文档已保存至: ${fullPath}`,
      });
    } catch (error) {
      toast({
        title: "导出失败",
        description: String(error),
        variant: "error",
      });
    }
  };

  const handleClear = () => {
    setLinksText("");
    clearLinks();
  };

  return (
    <div className="max-w-4xl mx-auto space-y-8">
      {/* 页面标题 */}
      <div>
        <h1 className="text-2xl font-semibold text-zinc-800 dark:text-zinc-100">
          抖音链接文案提取
        </h1>
        <p className="text-zinc-500 dark:text-zinc-400 mt-1.5 text-[15px]">
          输入抖音视频链接，批量提取视频文案
        </p>
      </div>

      {/* 服务状态提示 */}
      {dyMcpHealthy === false && (
        <div className="flex items-center gap-2 p-3 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800">
          <AlertCircle className="w-4 h-4 text-amber-500" />
          <span className="text-[13px] text-amber-700 dark:text-amber-300">
            dy-mcp 服务未运行，请先启动服务后再进行链接解析
          </span>
          <Button
            size="sm"
            variant="ghost"
            className="ml-auto text-amber-600 hover:text-amber-700"
            onClick={checkServicesHealth}
          >
            <RefreshCw className="w-3.5 h-3.5 mr-1" />
            重新检测
          </Button>
        </div>
      )}

      {/* 链接输入区域 */}
      <div className="bg-white dark:bg-zinc-900/50 rounded-2xl border border-zinc-200/80 dark:border-zinc-800/80 p-5 space-y-4">
        <div className="flex items-center gap-2 mb-2">
          <div className="w-8 h-8 rounded-lg bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center">
            <Link className="w-4 h-4 text-[#1976D2]" />
          </div>
          <span className="font-medium text-zinc-800 dark:text-zinc-100">
            输入链接
          </span>
        </div>

        <Textarea
          className="min-h-[160px] font-mono text-[13px]"
          placeholder="直接粘贴抖音分享内容，每行一条，例如：&#10;7.43 复制打开抖音，看看【酱紫的娱圈的作品】原来真的有人碰到这种情况会下车... https://v.douyin.com/Urqmkj6ZfWI/ 12/28&#10;3.21 复制打开抖音，看看【xxx的作品】这个视频太有意思了... https://v.douyin.com/xxxxx/"
          value={linksText}
          onChange={(e) => setLinksText(e.target.value)}
          disabled={isProcessing}
        />

        <div className="flex items-center justify-between pt-2">
          <p className="text-[13px] text-zinc-400">
            已输入{" "}
            <span className="text-[#1976D2] font-medium">{linkCount}</span>{" "}
            个链接
          </p>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleClear}
              className="rounded-lg"
              disabled={isProcessing}
            >
              清空
            </Button>
            <Button
              size="sm"
              className="rounded-lg gradient-primary text-white border-0"
              disabled={linkCount === 0 || isProcessing}
              onClick={handleStartParse}
            >
              {isProcessing ? (
                <>
                  <Loader2 className="w-4 h-4 mr-1.5 animate-spin" />
                  解析中...
                </>
              ) : (
                <>
                  <Play className="w-4 h-4 mr-1.5" />
                  开始解析
                </>
              )}
            </Button>
          </div>
        </div>
      </div>

      {/* 进度显示 */}
      {isProcessing && progress && (
        <div className="bg-white dark:bg-zinc-900/50 rounded-xl border border-zinc-200/80 dark:border-zinc-800/80 p-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-[13px] text-zinc-500">
              正在解析第 {progress.current} / {progress.total} 个链接
            </span>
            <span className="text-[13px] text-zinc-400">
              成功 {progress.success} / 失败 {progress.failed}
            </span>
          </div>
          <div className="w-full h-2 bg-zinc-100 dark:bg-zinc-800 rounded-full overflow-hidden">
            <div
              className="h-full bg-[#1976D2] transition-all duration-300"
              style={{
                width: `${(progress.current / progress.total) * 100}%`,
              }}
            />
          </div>
        </div>
      )}

      {/* 结果列表 */}
      {links.length > 0 && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-medium text-zinc-800 dark:text-zinc-100">
              解析结果
              <span className="ml-2 text-sm font-normal text-zinc-400">
                ({successCount}/{links.length} 成功)
              </span>
            </h2>
            <div className="flex gap-2">
              {failedCount > 0 && (
                <Button
                  size="sm"
                  variant="outline"
                  className="rounded-lg"
                  onClick={handleRetry}
                  disabled={isProcessing}
                >
                  <RefreshCw className="w-4 h-4 mr-1.5" />
                  重试失败 ({failedCount})
                </Button>
              )}
              {successCount > 0 && (
                <Button
                  size="sm"
                  className="rounded-lg"
                  onClick={handleExportDoc}
                  disabled={isProcessing}
                >
                  <FileText className="w-4 h-4 mr-1.5" />
                  导出文档
                </Button>
              )}
            </div>
          </div>

          {/* 统计信息 */}
          {stats && (
            <div className="grid grid-cols-3 gap-4">
              <div className="bg-zinc-50 dark:bg-zinc-800/50 rounded-lg p-3 text-center">
                <p className="text-2xl font-semibold text-zinc-800 dark:text-zinc-100">
                  {stats.total}
                </p>
                <p className="text-[13px] text-zinc-500">总计</p>
              </div>
              <div className="bg-emerald-50 dark:bg-emerald-900/20 rounded-lg p-3 text-center">
                <p className="text-2xl font-semibold text-emerald-600 dark:text-emerald-400">
                  {stats.success}
                </p>
                <p className="text-[13px] text-emerald-600 dark:text-emerald-400">
                  成功
                </p>
              </div>
              <div className="bg-red-50 dark:bg-red-900/20 rounded-lg p-3 text-center">
                <p className="text-2xl font-semibold text-red-600 dark:text-red-400">
                  {stats.failed}
                </p>
                <p className="text-[13px] text-red-600 dark:text-red-400">
                  失败
                </p>
              </div>
            </div>
          )}

          {/* 链接列表 */}
          <div className="space-y-2">
                {links.map((link) => (
                  <LinkResultCard
                    key={link.id}
                    link={link}
                    onChat={handleChat}
                    onSendToAgent={handleSendToAgent}
                  />
                ))}
          </div>
        </div>
      )}

      {/* 空状态 */}
      {links.length === 0 && (
        <div className="empty-state py-16">
          <div className="w-20 h-20 rounded-2xl bg-zinc-100 dark:bg-zinc-800/50 flex items-center justify-center mb-4">
            <Link className="w-10 h-10 text-zinc-300 dark:text-zinc-600" />
          </div>
          <p className="text-zinc-400 dark:text-zinc-500 text-[15px]">
            输入抖音链接开始提取文案
          </p>
        </div>
      )}
      {/* AI Chat Modal */}
      <AIChatModal
        isOpen={chatOpen}
        onClose={() => setChatOpen(false)}
        initialContext={chatContext}
        title={chatTitle}
      />
    </div>
  );
}
