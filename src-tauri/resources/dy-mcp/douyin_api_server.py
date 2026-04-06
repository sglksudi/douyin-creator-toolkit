#!/usr/bin/env python3
"""
Douyin Link Parser - Simple HTTP API Server
用于抖音链接解析的简单 HTTP API 服务器
"""

import re
import json
import os
from http.server import HTTPServer, BaseHTTPRequestHandler
import urllib.parse
import requests
from pathlib import Path

HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) "
        "AppleWebKit/605.1.15 (KHTML, like Gecko) EdgiOS/121.0.2277.107 "
        "Version/17.0 Mobile/15E148 Safari/604.1"
    ),
    "Accept": "text/html,application/xhtml+xml",
    "Accept-Language": "zh-CN,zh;q=0.9",
}


def parse_douyin_link(share_text: str) -> dict:
    """解析抖音分享链接，返回视频信息"""
    # 1. 提取 URL
    urls = re.findall(r'http[s]?://[^\s]+', share_text)
    if not urls:
        raise ValueError("未找到有效的分享链接")

    share_url = urls[0]
    
    # 2. 跟随重定向获取视频 ID
    resp = requests.get(share_url, headers=HEADERS, timeout=10)
    video_id = resp.url.split("?")[0].strip("/").split("/")[-1]
    
    # 3. 访问分享页面
    share_page_url = f"https://www.iesdouyin.com/share/video/{video_id}"
    response = requests.get(share_page_url, headers=HEADERS, timeout=15)
    response.raise_for_status()

    # 4. 解析 _ROUTER_DATA
    pattern = re.compile(r"window\._ROUTER_DATA\s*=\s*(.*?)</script>", flags=re.DOTALL)
    find_res = pattern.search(response.text)
    if not find_res or not find_res.group(1):
        raise ValueError("解析视频信息失败：未找到 _ROUTER_DATA")

    json_data = json.loads(find_res.group(1).strip())
    
    # 5. 提取视频数据
    VIDEO_ID_PAGE_KEY = "video_(id)/page"
    NOTE_ID_PAGE_KEY = "note_(id)/page"

    if VIDEO_ID_PAGE_KEY in json_data.get("loaderData", {}):
        data = json_data["loaderData"][VIDEO_ID_PAGE_KEY]["videoInfoRes"]["item_list"][0]
    elif NOTE_ID_PAGE_KEY in json_data.get("loaderData", {}):
        data = json_data["loaderData"][NOTE_ID_PAGE_KEY]["videoInfoRes"]["item_list"][0]
    else:
        raise ValueError("无法解析视频信息：找不到 loaderData")

    # 6. 提取字段
    video_url = data["video"]["play_addr"]["url_list"][0].replace("playwm", "play")
    title = data.get("desc", "").strip() or f"douyin_{video_id}"
    title = re.sub(r'[\\/:*?"<>|]', '_', title)
    
    author = data.get("author", {}).get("nickname", "未知")
    statistics = data.get("statistics", {})
    
    return {
        "video_id": video_id,
        "title": title,
        "author": author,
        "download_url": video_url,
        "likes": statistics.get("digg_count", 0),
        "comments": statistics.get("comment_count", 0),
        "shares": statistics.get("share_count", 0),
        "cover": data.get("video", {}).get("cover", {}).get("url_list", [""])[0],
    }


class DouyinAPIHandler(BaseHTTPRequestHandler):
    """HTTP 请求处理器"""

    def _require_sidecar_token(self):
        expected_token = SIDECAR_TOKEN
        if not expected_token:
            self.send_json({"status": "error", "error": "sidecar token 未配置"}, 500)
            return False

        request_token = self.headers.get("X-Sidecar-Token", "")
        if request_token != expected_token:
            self.send_json({"status": "error", "error": "未授权的请求"}, 403)
            return False

        return True

    def _resolve_download_path(self, raw_path: str) -> Path:
        candidate = Path(raw_path)
        if not candidate.is_absolute():
            raise ValueError("下载路径必须是绝对路径")

        resolved_candidate = candidate.resolve(strict=False)
        allowed_root = SIDECAR_DOWNLOAD_ROOT.resolve(strict=False)

        try:
            resolved_candidate.relative_to(allowed_root)
        except ValueError as exc:
            raise ValueError("下载路径超出允许目录") from exc

        return resolved_candidate
    
    def do_GET(self):
        """处理 GET 请求"""
        parsed = urllib.parse.urlparse(self.path)
        
        if parsed.path == "/health":
            # 健康检查
            self.send_json({"status": "ok"})
            
        elif parsed.path == "/parse":
            if not self._require_sidecar_token():
                return
            # 解析链接
            query = urllib.parse.parse_qs(parsed.query)
            link = query.get("link", [""])[0]
            
            if not link:
                self.send_json({"status": "error", "error": "缺少 link 参数"}, 400)
                return
            
            try:
                result = parse_douyin_link(link)
                self.send_json({"status": "success", "data": result})
            except Exception as e:
                self.send_json({"status": "error", "error": str(e)}, 500)
        else:
            self.send_json({"status": "error", "error": "未知路径"}, 404)
    
    def do_POST(self):
        """处理 POST 请求"""
        if self.path == "/parse":
            self.handle_parse()
        elif self.path == "/download":
            self.handle_download()
        else:
            self.send_json({"status": "error", "error": "未知路径"}, 404)

    def handle_parse(self):
        if not self._require_sidecar_token():
            return

        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length).decode("utf-8")
        
        try:
            data = json.loads(body) if body else {}
            link = data.get("link", "")
        except json.JSONDecodeError:
            link = body
        
        if not link:
            self.send_json({"status": "error", "error": "缺少 link 参数"}, 400)
            return
        
        try:
            result = parse_douyin_link(link)
            self.send_json({"status": "success", "data": result})
        except Exception as e:
            self.send_json({"status": "error", "error": str(e)}, 500)

    def handle_download(self):
        if not self._require_sidecar_token():
            return

        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length).decode("utf-8")
        
        try:
            data = json.loads(body)
            url = data.get("url", "")
            save_path = data.get("path", "")
        except Exception:
            self.send_json({"status": "error", "error": "无效的 JSON 参数"}, 400)
            return

        if not url or not save_path:
            self.send_json({"status": "error", "error": "缺少 url 或 path 参数"}, 400)
            return

        try:
            resolved_path = self._resolve_download_path(save_path)
            resolved_path.parent.mkdir(parents=True, exist_ok=True)

            # 使用流式下载
            print(f"[Download] Downloading to {resolved_path}")
            response = requests.get(url, headers=HEADERS, stream=True, timeout=60)
            response.raise_for_status()
            
            with open(resolved_path, 'wb') as f:
                for chunk in response.iter_content(chunk_size=8192):
                    if chunk:
                        f.write(chunk)
                        
            self.send_json({"status": "success"})
        except Exception as e:
            print(f"[Download] Error: {e}")
            self.send_json({"status": "error", "error": str(e)}, 500)

    def send_json(self, data, status=200):
        """发送 JSON 响应"""
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.end_headers()
        self.wfile.write(json.dumps(data, ensure_ascii=False).encode("utf-8"))
    
    def log_message(self, format, *args):
        """简化日志输出"""
        print(f"[API] {args[0]}")

# 配置
SIDECAR_PORT = int(os.environ.get("SIDECAR_PORT", "38080"))
SIDECAR_HOST = os.environ.get("SIDECAR_HOST", "127.0.0.1")
SIDECAR_TOKEN = os.environ.get("SIDECAR_TOKEN", "")
SIDECAR_DOWNLOAD_ROOT = Path(
    os.environ.get("SIDECAR_DOWNLOAD_ROOT", os.path.join(os.getcwd(), "downloads"))
)


def main():
    """启动服务器"""
    server = HTTPServer((SIDECAR_HOST, SIDECAR_PORT), DouyinAPIHandler)
    print(f"[Douyin API] 服务器启动在 http://{SIDECAR_HOST}:{SIDECAR_PORT}")
    print(f"[Douyin API] 解析接口: GET /parse?link=<抖音链接>")
    print(f"[Douyin API] 健康检查: GET /health")
    server.serve_forever()


if __name__ == "__main__":
    main()
