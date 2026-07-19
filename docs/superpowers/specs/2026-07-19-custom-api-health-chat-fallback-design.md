# Custom API Health Chat Fallback Design

## Goal

Make custom OpenAI-compatible API detection work for providers that support `/chat/completions` but do not expose `/models`.

## Current Behavior

`CustomApiProvider::is_available()` sends `GET <root>/models` and returns `true` only for a successful status. Some compatible gateways reject, hide, or do not implement `/models`, even though normal chat requests work.

## Design

The health check remains backend-only and keeps the existing Tauri command contract: `check_custom_api_provider(provider) -> Result<bool, String>`.

`CustomApiProvider::is_available()` will:

1. Send the existing authenticated `GET /models` request with the current 10 second timeout.
2. Return `Ok(true)` when `/models` succeeds.
3. Return `Err(AiError::InvalidApiKey)` when `/models` returns `401`, without fallback.
4. For other non-success `/models` responses, send a lightweight authenticated `POST /chat/completions` ping.
5. Return `Ok(true)` when the chat ping returns a parseable OpenAI-compatible response.
6. Return `Ok(false)` when the chat ping returns a non-success HTTP status.
7. Preserve network and parse errors as errors so the UI still reports a detection failure instead of a false success.

The chat ping uses the configured model and a single short user message: `ping`. No frontend changes are required.

## Testing

Rust unit tests will use a small local `TcpListener` server rather than adding a new dependency. Tests cover:

- `/models` success short-circuits and does not call chat.
- `/models` 404 falls back to chat and succeeds when chat returns OpenAI-compatible JSON.
- `/models` 401 does not fallback and maps to `InvalidApiKey`.
- `/models` 404 plus chat HTTP failure returns `Ok(false)`.

## Scope

This does not change provider storage, settings UI, built-in provider checks, or runtime AI chat behavior.
