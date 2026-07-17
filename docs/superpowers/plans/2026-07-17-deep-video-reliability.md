# Deep Video Reliability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Deep Video task state authoritative and recoverable, synchronize AI settings correctly, and restore the complete test suite.

**Architecture:** Keep the existing ID-based task queue. Add a small abort-handle registry at the Deep Video command boundary, make queue transitions gate terminal events, and keep frontend listeners alive at `App` scope with backend reconciliation. Reuse existing settings and retry paths.

**Tech Stack:** Rust, Tauri 2, futures-util, React 19, Zustand 5, TypeScript 5.8.

## Global Constraints

- Do not replace `TaskQueue` with a new scheduler.
- Do not implement cooperative pause/resume for frame extraction.
- Do not add a frontend test framework or another dependency.
- Do not delete partial artifacts left by a cancelled task.
- Preserve unrelated files and existing user changes.

---

### Task 1: Restore the Rust Test Baseline

**Files:**
- Modify: `src-tauri/tests/config_properties.rs:108`
- Modify: `src-tauri/tests/config_properties.rs:233`

**Interfaces:**
- Consumes: `AppConfig { custom_api_providers: Vec<CustomApiProviderConfig> }`.
- Produces: a compiling integration-test target with explicit empty provider lists in legacy strategies.

- [ ] **Step 1: Reproduce the failing test target**

Run: `cd src-tauri && cargo test --test config_properties --no-run`
Expected: `E0063` at lines 108 and 233 for missing `custom_api_providers`.

- [ ] **Step 2: Add the new field to both test fixtures**

```rust
custom_api_providers: Vec::new(),
```

- [ ] **Step 3: Verify the property target**

Run: `cd src-tauri && cargo test --test config_properties`
Expected: all `config_properties` tests pass.

### Task 2: Make Deep Video Task Transitions Authoritative

**Files:**
- Modify: `src-tauri/src/data/task_queue.rs:495-559`
- Modify: `src-tauri/src/commands/deep_video.rs:1-80`
- Modify: `src-tauri/src/commands/task_queue.rs:155-269`
- Modify: `src/pages/TaskHistory.tsx:108-124`

**Interfaces:**
- Consumes: `TaskQueue::{start_task_by_id, update_task_progress_by_id, complete_task_by_id, fail_task_by_id, cancel_task}`.
- Produces: `abort_deep_video_task(task_id: &str) -> bool` and a `task-cancelled` Tauri event with `{ task_id }`.

- [ ] **Step 1: Add failing queue tests**

```rust
#[tokio::test]
async fn clear_pending_keeps_id_started_tasks() {
    let queue = TaskQueue::new();
    let running = queue
        .add_task(TaskType::AiAnalysis {
            content: "running".to_string(),
            video_id: "running".to_string(),
        })
        .await;
    let pending = queue
        .add_task(TaskType::AiAnalysis {
            content: "pending".to_string(),
            video_id: "pending".to_string(),
        })
        .await;
    queue.start_task_by_id(&running).await.unwrap();
    queue.clear_pending().await;
    assert!(queue.get_task(&running).await.is_some());
    assert!(queue.get_task(&pending).await.is_none());
}

#[tokio::test]
async fn id_started_task_persists_progress() {
    let queue = TaskQueue::new();
    let id = queue
        .add_task(TaskType::AiAnalysis {
            content: "progress".to_string(),
            video_id: "progress".to_string(),
        })
        .await;
    queue.start_task_by_id(&id).await.unwrap();
    queue.update_task_progress_by_id(&id, 0.1).await;
    assert_eq!(queue.get_task(&id).await.unwrap().progress, 0.1);
}
```

- [ ] **Step 2: Run the focused tests and observe the clear failure**

Run: `cd src-tauri && cargo test data::task_queue::tests::`
Expected: the clear test fails because the running deque item is removed.

- [ ] **Step 3: Retain non-pending tasks**

```rust
pub async fn clear_pending(&self) {
    self.tasks
        .write()
        .await
        .retain(|task| !matches!(task.status, TaskStatus::Pending));
}
```

- [ ] **Step 4: Add abort ownership and gate terminal events**

Use `futures_util::future::{AbortHandle, Abortable}` and a `Lazy<Mutex<HashMap<String, AbortHandle>>>`. Insert the handle before returning the task ID, wrap `run_deep_video_analysis` in `Abortable`, remove the handle at exit, and emit completion/failure only when the queue transition returns `Some`.

```rust
if get_task_queue()
    .complete_task_by_id(&task_id_for_task, Some(result_path.clone()))
    .await
    .is_some()
{
    emit_task_completed(&app_for_task, &task_id_for_task, Some(&result_path));
}
```

- [ ] **Step 5: Wire cancellation and persist initial progress**

Change `cancel_task` to accept `AppHandle`, transition the queue first, call `abort_deep_video_task`, then emit `task-cancelled`. Before `emit_task_progress(..., 0.1, ...)`, call `update_task_progress_by_id(..., 0.1).await`.

- [ ] **Step 6: Stop advertising pause for Deep Video**

```tsx
{task.status === 'running' && task.task_type !== 'deep_video_analysis' && onPause && (
  <Button variant="ghost" size="icon" onClick={onPause}>
    <Pause className="h-4 w-4" />
  </Button>
)}
```

- [ ] **Step 7: Run backend tests**

Run: `cd src-tauri && cargo test --lib`
Expected: all library tests pass, including the two new queue tests.

### Task 3: Replace AI Runtime Settings Instead of Merging Them

**Files:**
- Modify: `src-tauri/src/commands/ai.rs:215-247`
- Modify: `src-tauri/src/commands/settings.rs:58-89`
- Modify: `src-tauri/src/commands/settings.rs:251-262`

**Interfaces:**
- Consumes: `AiSettings` and `AiService`.
- Produces: `apply_ai_settings(service: &mut AiService, settings: AiSettings)` with replacement semantics.

- [ ] **Step 1: Add a failing replacement test**

```rust
#[test]
fn applying_settings_clears_existing_api_keys() {
    let mut service = AiService::new();
    service.openai_api_key = Some("old".into());
    apply_ai_settings(&mut service, AiSettings::default());
    assert_eq!(service.openai_api_key, None);
}
```

- [ ] **Step 2: Run the focused test**

Run: `cd src-tauri && cargo test commands::ai::tests::applying_settings_clears_existing_api_keys`
Expected: fail until `apply_ai_settings` exists and replaces optional keys.

- [ ] **Step 3: Implement one shared replacement helper**

```rust
pub(crate) fn apply_ai_settings(service: &mut AiService, settings: AiSettings) {
    service.set_custom_api_providers(settings.custom_api_providers);
    service.set_provider_from_key(settings.provider);
    service.doubao_api_key = settings.doubao_api_key.filter(|key| !key.trim().is_empty());
    service.openai_api_key = settings.openai_api_key.filter(|key| !key.trim().is_empty());
    service.deepseek_api_key = settings.deepseek_api_key.filter(|key| !key.trim().is_empty());
    service.lm_studio_url = settings.lm_studio_url;
}
```

Call it from both `update_ai_settings` and startup restoration. After `reset_to_default`, apply the loaded default config before returning it.

- [ ] **Step 4: Verify settings tests**

Run: `cd src-tauri && cargo test commands::ai data::config`
Expected: all selected tests pass.

### Task 4: Keep Deep Video State Synchronized Across Routes

**Files:**
- Modify: `src/App.tsx:70-92`
- Modify: `src/pages/LocalVideo.tsx:40-75`
- Modify: `src/pages/DouyinLink.tsx:350-401`
- Modify: `src/stores/useVideoStore.ts:50-566`
- Modify: `src/stores/useDouyinLinkStore.ts:35-564`

**Interfaces:**
- Consumes: `setupProgressListener() -> Promise<() => void>` and `get_task_info`.
- Produces: `DeepAnalysisStatus` including `cancelled`, global listener lifetime, and reconciliation of every stored running task.

- [ ] **Step 1: Extend terminal state and event types**

```ts
type DeepAnalysisStatus = "idle" | "running" | "completed" | "failed" | "cancelled";
interface TaskCancelledEvent { task_id: string }
```

- [ ] **Step 2: Register and handle `task-cancelled` in both stores**

Set matching analysis state to `{ ...deepAnalysis, status: "cancelled" }`. Extend reconciliation to accept `completed`, `failed`, and `cancelled`.

- [ ] **Step 3: Reconcile existing running items after listener setup**

```ts
await Promise.all(
  get().videos
    .filter((video) => video.deepAnalysis?.status === "running" && video.deepAnalysis.taskId)
    .map((video) => reconcileDeepAnalysisTask(
      video.id,
      video.deepAnalysis!.taskId!,
      Boolean(video.deepAnalysis!.useFrameAnalysis)
    ))
);
```

Use the equivalent link scan in `useDouyinLinkStore`.

- [ ] **Step 4: Move listener ownership to `App`**

In one app-level effect, call both store setup functions, track a `disposed` flag, immediately invoke late cleanup functions after disposal, and remove the page-level listener effects.

- [ ] **Step 5: Add store-level duplicate-start guards**

At the start of each action, read the latest matching item and return when `deepAnalysis?.status === "running"`.

- [ ] **Step 6: Verify TypeScript**

Run: `npm.cmd run build`
Expected: TypeScript and Vite production build pass.

### Task 5: Restore Reachable Retries and Item-Specific Frame Mode

**Files:**
- Modify: `src/stores/useDouyinLinkStore.ts:304-312`
- Modify: `src/stores/useVideoStore.ts:45-160`
- Modify: `src/pages/LocalVideo.tsx:60-180`
- Modify: `src/pages/LocalVideo.tsx:612-640`

**Interfaces:**
- Consumes: existing `retryFailedLinks`, `VideoItem`, and `startDeepAnalysis`.
- Produces: failed transcript extraction that enters the existing retry path and per-video `useFrameAnalysis` selection.

- [ ] **Step 1: Make extraction failure retryable**

```ts
{ ...link, status: "failed" as const, error: `Transcript extraction failed: ${e}` }
```

- [ ] **Step 2: Store frame mode on each video**

Add `useFrameAnalysis?: boolean` to `VideoItem` and `setUseFrameAnalysis(id, enabled)` to the store. Bind each switch to its video and pass `Boolean(video.useFrameAnalysis)` when starting analysis.

- [ ] **Step 3: Verify frontend build**

Run: `npm.cmd run build`
Expected: production build passes.

- [ ] **Step 4: Run the full verification matrix**

Run: `cd src-tauri && cargo test`
Expected: all Rust unit, integration, and property tests pass.

Run: `npm.cmd run build`
Expected: TypeScript and Vite build pass.

Run: `git diff --check`
Expected: no output.

- [ ] **Step 5: Perform Tauri smoke checks**

Verify one local analysis completes after switching routes, one analysis can be cancelled and restarted, two rapid starts create one task, and one simulated/external transcript failure exposes the retry action.
