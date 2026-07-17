# Deep Video Reliability Design

## Goal

Close the confirmed task lifecycle, settings synchronization, and frontend recovery bugs without replacing the existing task queue or adding dependencies.

## Scope

- Restore compilation of the full Rust test suite and extend the affected configuration property coverage.
- Make Deep Video cancellation stop the spawned analysis task and emit one authoritative cancelled terminal state.
- Keep ID-based concurrent Deep Video tasks in the existing queue while ensuring `clear_pending` removes only pending tasks.
- Do not expose pause/resume controls for Deep Video until the runner supports cooperative pausing.
- Persist emitted Deep Video progress in the task queue.
- Register Local Video and Douyin Link progress listeners for the lifetime of `App`, then reconcile stored running task IDs against backend state.
- Represent `cancelled` in both source stores and allow a cancelled analysis to be started again.
- Reject duplicate starts for an item while its analysis is already running.
- Synchronize AI service credentials and provider state by replacement, including key clearing and settings reset.
- Mark failed Douyin transcript extraction as failed so the existing retry workflow can reach it.

## Backend Design

Deep Video command execution remains a spawned async task. A small task-handle registry keyed by task ID owns cancellation of those spawned tasks. The cancel command first records the queue transition, aborts a matching Deep Video handle, and emits `task-cancelled`. Completion and failure events are emitted only when the corresponding queue transition succeeds, preventing a cancelled task from later publishing a contradictory terminal event.

`TaskQueue::clear_pending` retains every non-pending task. ID-based progress updates continue to use the existing `update_task_progress_by_id` method. Pause and resume behavior for other task types remains unchanged; the frontend suppresses those controls for Deep Video.

AI settings updates assign all optional keys and the LM Studio URL directly, instead of treating missing or empty values as "leave unchanged". Resetting settings applies the resulting default configuration to the global AI service before returning it.

## Frontend Design

`App` owns the two existing store listener registrations, so route changes cannot create an event gap. Listener setup also reconciles every stored Deep Video item that is still marked running and has a task ID. Both stores accept `cancelled` as a terminal, retryable state and handle the new cancellation event.

Store actions check the latest state before invoking the backend. A second start for an already-running item returns without creating another task. Douyin transcript extraction failures set the item status to `failed`, reusing the existing retry action and button.

The Local Video frame-evidence selection becomes item-specific because the control is rendered inside each item. No new global setting is introduced.

## Error Handling

- Aborting a Deep Video task may leave partial frame artifacts, but no final result or completion event is published. Partial artifact cleanup is intentionally deferred because deleting files during an external process shutdown can race on Windows.
- Missing task IDs during completion/failure are treated as stale or cancelled work and produce no terminal event.
- Listener setup failures remain logged; successful partial registrations are cleaned up by the returned cleanup function.

## Verification

- Rust unit tests cover pending-only clearing, ID-based progress, cancellation transition behavior, AI key replacement, and reset application where practical.
- Configuration property tests compile and include `custom_api_providers` in generated values or explicit defaults.
- `cargo test`, `npm.cmd run build`, and `git diff --check` must pass.
- Tauri smoke verification covers start, route switch, completion recovery, cancellation, retry, and duplicate-start prevention.

## Non-Goals

- Replacing `TaskQueue` with a new scheduler.
- Implementing cooperative pause/resume for frame extraction.
- Adding a frontend test framework.
- Deleting partial artifacts left by a cancelled task.
