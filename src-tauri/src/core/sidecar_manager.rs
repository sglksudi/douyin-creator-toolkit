use once_cell::sync::OnceCell;
use std::ffi::OsString;
use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Manager};
use tracing::{error, info};
use uuid::Uuid;

// Windows creating process without window flag
const CREATE_NO_WINDOW: u32 = 0x08000000;

// GPU ASR 服务端口
pub const ASR_GPU_PORT: u16 = 38081;

static SIDECAR_TOKEN: OnceCell<String> = OnceCell::new();

#[derive(Debug, PartialEq, Eq)]
enum AsrServiceStatus {
    Healthy,
    Unavailable,
}

pub struct SidecarState {
    pub child: Mutex<Option<Child>>,
    pub asr_gpu_child: Mutex<Option<Child>>,
}

impl Default for SidecarState {
    fn default() -> Self {
        Self {
            child: Mutex::new(None),
            asr_gpu_child: Mutex::new(None),
        }
    }
}

pub fn sidecar_token() -> Option<String> {
    SIDECAR_TOKEN.get().cloned()
}

pub fn init_sidecar(app: &AppHandle) {
    let state = app.state::<SidecarState>();
    let resource_dir = app
        .path()
        .resource_dir()
        .expect("Failed to get resource dir");
    let download_root = std::env::temp_dir().join("douyin_creator_tools");

    if let Err(e) = std::fs::create_dir_all(&download_root) {
        error!(
            "[Sidecar] Failed to prepare download root {:?}: {}",
            download_root, e
        );
        return;
    }

    let sidecar_token = SIDECAR_TOKEN
        .get_or_init(|| Uuid::new_v4().to_string())
        .clone();

    // 尝试多个 Python 路径（按优先级）
    // 生产环境优先使用嵌入式 Python，开发环境可以用虚拟环境
    let python_candidates = [
        // 1. 优先使用嵌入式 Python (真正可移植)
        resource_dir
            .join("resources")
            .join("python-embed")
            .join("python.exe"),
        // 2. 备选：直接在 resources 下的嵌入式 Python
        resource_dir.join("python-embed").join("python.exe"),
        // 3. 开发模式：虚拟环境 (仅在开发机上可用)
        resource_dir
            .join("resources")
            .join("python-env")
            .join("Scripts")
            .join("python.exe"),
        // 4. 回退到系统 Python
        std::path::PathBuf::from("python"),
        std::path::PathBuf::from("py"),
    ];

    let python_path = python_candidates
        .iter()
        .find(|p| is_python_available(p))
        .cloned();

    let python_path = match python_path {
        Some(p) => p,
        None => {
            eprintln!("[Sidecar] Error: Python executable not found in any candidate path");
            eprintln!("[Sidecar] Searched paths:");
            for p in &python_candidates {
                eprintln!("[Sidecar]   - {:?}", p);
            }
            return;
        }
    };

    // 脚本路径 - 同样尝试多个位置
    let script_candidates = [
        resource_dir
            .join("resources")
            .join("dy-mcp")
            .join("douyin_api_server.py"),
        resource_dir.join("dy-mcp").join("douyin_api_server.py"),
    ];

    let script_path = script_candidates.iter().find(|p| p.exists()).cloned();

    let script_path = match script_path {
        Some(p) => p,
        None => {
            eprintln!("[Sidecar] Error: API Script not found");
            eprintln!("[Sidecar] Searched paths:");
            for p in &script_candidates {
                eprintln!("[Sidecar]   - {:?}", p);
            }
            return;
        }
    };

    eprintln!("[Sidecar] Python Path: {:?}", python_path);
    eprintln!("[Sidecar] Script Path: {:?}", script_path);

    // Spawn process
    let mut command = Command::new(&python_path);
    command
        .arg(&script_path)
        .env("SIDECAR_TOKEN", &sidecar_token)
        .env("SIDECAR_DOWNLOAD_ROOT", &download_root)
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8")
        .current_dir(&resource_dir) // Set working directory
        .creation_flags(CREATE_NO_WINDOW) // Hide console window
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(runtime_path) = build_runtime_path_env(&resource_dir) {
        command.env("PATH", runtime_path);
    }

    match command.spawn() {
        Ok(mut child) => {
            let mcp_pid = child.id();
            info!("[Sidecar] Started Python API Server with PID: {}", mcp_pid);

            // 处理 stdout
            if let Some(stdout) = child.stdout.take() {
                thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            info!("[Python] {}", line);
                        }
                    }
                });
            }

            // 处理 stderr
            if let Some(stderr) = child.stderr.take() {
                thread::spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            error!("[Python Error] {}", line);
                        }
                    }
                });
            }

            // 启动 GPU ASR 服务
            let asr_gpu_child = start_asr_gpu_server(&python_path, &resource_dir);

            *state.child.lock().expect("sidecar child mutex poisoned") = Some(child);
            *state
                .asr_gpu_child
                .lock()
                .expect("sidecar gpu child mutex poisoned") = asr_gpu_child;
        }
        Err(e) => {
            eprintln!("[Sidecar] Failed to start Python API Server: {}", e);
            eprintln!("[Sidecar] Python: {:?}", python_path);
            eprintln!("[Sidecar] Script: {:?}", script_path);
        }
    }
}

fn is_python_available(path: &Path) -> bool {
    if path.is_absolute() && !path.exists() {
        return false;
    }

    Command::new(path)
        .arg("--version")
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn build_runtime_path_env(resource_dir: &Path) -> Option<OsString> {
    let mut runtime_dirs = runtime_search_dirs(resource_dir);

    if let Some(existing_path) = std::env::var_os("PATH") {
        runtime_dirs.extend(std::env::split_paths(&existing_path));
    }

    std::env::join_paths(runtime_dirs).ok()
}

fn runtime_search_dirs(resource_dir: &Path) -> Vec<PathBuf> {
    [
        resource_dir.join("resources").join("bin"),
        resource_dir.join("bin"),
        resource_dir.join("resources").join("python-embed"),
        resource_dir.join("python-embed"),
        resource_dir.join("resources").join("dy-mcp"),
        resource_dir.join("dy-mcp"),
    ]
    .into_iter()
    .filter(|path| path.exists())
    .collect()
}

/// Start the GPU ASR sidecar service
fn start_asr_gpu_server(python_path: &Path, resource_dir: &Path) -> Option<Child> {
    if asr_service_status(ASR_GPU_PORT) == AsrServiceStatus::Healthy {
        info!(
            "[ASR-GPU] Reusing existing ASR service at http://127.0.0.1:{}",
            ASR_GPU_PORT
        );
        return None;
    }

    // Resolve the GPU ASR script path
    let asr_script_candidates = [
        resource_dir
            .join("resources")
            .join("dy-mcp")
            .join("asr_gpu_server.py"),
        resource_dir.join("dy-mcp").join("asr_gpu_server.py"),
    ];

    let asr_script = asr_script_candidates.iter().find(|p| p.exists()).cloned();

    let asr_script = match asr_script {
        Some(p) => p,
        None => {
            eprintln!("[ASR-GPU] GPU ASR script not found, GPU acceleration disabled");
            return None;
        }
    };

    eprintln!("[ASR-GPU] Starting GPU ASR Server...");
    eprintln!("[ASR-GPU] Script: {:?}", asr_script);

    let mut command = Command::new(python_path);
    command
        .arg(&asr_script)
        .env("ASR_GPU_PORT", ASR_GPU_PORT.to_string())
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8")
        .current_dir(resource_dir)
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(runtime_path) = build_runtime_path_env(resource_dir) {
        command.env("PATH", runtime_path);
    }

    match command.spawn() {
        Ok(mut child) => {
            info!("[ASR-GPU] Started GPU ASR Server with PID: {}", child.id());
            info!("[ASR-GPU] Service URL: http://127.0.0.1:{}", ASR_GPU_PORT);

            // 处理 stdout
            if let Some(stdout) = child.stdout.take() {
                thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            info!("[ASR-GPU Python] {}", line);
                        }
                    }
                });
            }

            // 处理 stderr
            if let Some(stderr) = child.stderr.take() {
                thread::spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            error!("[ASR-GPU Error] {}", line);
                        }
                    }
                });
            }

            Some(child)
        }
        Err(e) => {
            eprintln!("[ASR-GPU] Failed to start GPU ASR Server: {}", e);
            None
        }
    }
}

fn asr_service_status(port: u16) -> AsrServiceStatus {
    let url = format!("http://127.0.0.1:{port}/health");
    let response = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(2))
        .call();

    match response {
        Ok(response) if response.status() == 200 => AsrServiceStatus::Healthy,
        _ => AsrServiceStatus::Unavailable,
    }
}

pub fn cleanup_sidecars(app: &AppHandle) {
    let state = app.state::<SidecarState>();
    terminate_child(
        &mut *state.child.lock().expect("sidecar child mutex poisoned"),
        "Python API Server",
    );
    terminate_child(
        &mut *state
            .asr_gpu_child
            .lock()
            .expect("sidecar gpu child mutex poisoned"),
        "GPU ASR Server",
    );
}

fn terminate_child(child_slot: &mut Option<Child>, process_name: &str) {
    let Some(child) = child_slot.as_mut() else {
        return;
    };

    match child.try_wait() {
        Ok(Some(status)) => {
            info!(
                "[Sidecar] {} already exited with status {}",
                process_name, status
            );
        }
        Ok(None) => {
            if let Err(e) = child.kill() {
                error!("[Sidecar] Failed to stop {}: {}", process_name, e);
            } else {
                let _ = child.wait();
                info!("[Sidecar] {} stopped", process_name);
            }
        }
        Err(e) => {
            error!("[Sidecar] Failed to query {} status: {}", process_name, e);
        }
    }

    *child_slot = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[test]
    fn detects_existing_healthy_asr_service() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 512];
            let _ = stream.read(&mut buffer);
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 35\r\n\r\n{\"status\":\"ok\",\"service\":\"asr-gpu\"}",
                )
                .unwrap();
        });

        assert_eq!(asr_service_status(port), AsrServiceStatus::Healthy);
    }
}
