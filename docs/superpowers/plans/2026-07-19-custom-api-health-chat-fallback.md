# Custom API Health Chat Fallback Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a chat-completions fallback to custom OpenAI-compatible API health checks when `/models` is unavailable.

**Architecture:** Keep the change in `CustomApiProvider`. `is_available()` remains the command-facing health check and delegates to a lightweight chat ping only after non-auth `/models` failures.

**Tech Stack:** Rust, reqwest, Tokio, existing unit tests in `src-tauri/src/ai/service.rs`.

## Global Constraints

- Do not change frontend settings UI or the Tauri command signature.
- Do not add new crates for HTTP mocking.
- Do not fallback on `/models` 401 responses.
- Use the configured custom provider model for the chat ping.

---

### Task 1: Custom Provider Health Fallback

**Files:**
- Modify: `src-tauri/src/ai/service.rs`
- Test: `src-tauri/src/ai/service.rs`

**Interfaces:**
- Consumes: `CustomApiProvider::models_url() -> String`, `CustomApiProvider::chat_completions_url() -> String`, `CustomApiProvider::with_auth(reqwest::RequestBuilder) -> reqwest::RequestBuilder`
- Produces: `CustomApiProvider::is_available() -> Result<bool, AiError>` with chat fallback behavior

- [ ] **Step 1: Write local HTTP test helper**

Add a test-only helper inside `#[cfg(test)] mod tests` that accepts a list of HTTP responses and records request paths. Use `std::net::TcpListener`, `std::thread`, and `std::sync::{Arc, Mutex}`.

- [ ] **Step 2: Write failing `/models` success test**

Add `custom_provider_health_uses_models_when_available`. It starts the local server with one `200 OK` JSON response for `/models`, calls `provider.is_available().await`, asserts `true`, and asserts only `/models` was requested.

Run: `cargo test ai::service::tests::custom_provider_health_uses_models_when_available --lib`

Expected before implementation cleanup: PASS or reveal helper issues. This behavior already exists, so it protects the short-circuit contract.

- [ ] **Step 3: Write failing fallback success test**

Add `custom_provider_health_falls_back_to_chat_when_models_missing`. It returns `404` for `/models`, then `200` with `{"choices":[{"message":{"role":"assistant","content":"pong"}}]}` for `/chat/completions`. Assert `is_available()` returns `true` and both paths were requested.

Run: `cargo test ai::service::tests::custom_provider_health_falls_back_to_chat_when_models_missing --lib`

Expected: FAIL because `is_available()` currently returns `Ok(false)` after `/models` 404.

- [ ] **Step 4: Write failing auth guard test**

Add `custom_provider_health_does_not_fallback_on_unauthorized_models`. It returns `401` for `/models`, calls `is_available()`, asserts `Err(AiError::InvalidApiKey)`, and asserts only `/models` was requested.

Run: `cargo test ai::service::tests::custom_provider_health_does_not_fallback_on_unauthorized_models --lib`

Expected: FAIL because current behavior returns `Ok(false)`.

- [ ] **Step 5: Write failing chat failure test**

Add `custom_provider_health_returns_false_when_chat_ping_fails`. It returns `404` for `/models`, then `500` for `/chat/completions`. Assert `Ok(false)` and both paths were requested.

Run: `cargo test ai::service::tests::custom_provider_health_returns_false_when_chat_ping_fails --lib`

Expected: PASS after fallback implementation, or FAIL before fallback if chat is never requested.

- [ ] **Step 6: Implement minimal production code**

Change `is_available()` to inspect `/models` status. On success return `Ok(true)`. On `401`, return `Err(AiError::InvalidApiKey)`. On other HTTP failures, call a new private async helper such as `chat_ping_available()` that posts a one-message `ChatRequest` to `chat_completions_url()` with a 10 second timeout and returns `Ok(true)` for parseable success, `Ok(false)` for non-success status.

- [ ] **Step 7: Verify focused tests**

Run: `cargo test ai::service::tests::custom_provider_health --lib`

Expected: all custom provider health tests pass.

- [ ] **Step 8: Verify broader backend tests**

Run: `cargo test --lib`

Expected: lib tests pass.

- [ ] **Step 9: Check whitespace and review diff**

Run: `git diff --check`
