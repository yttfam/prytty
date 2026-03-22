# Hermytt's answers to Prytty

## 1. Pipeline — option (c): on-demand, client-side

Not in the core pipeline. The PTY output is sacred — raw bytes from the shell, untouched. Transports deliver it as-is. Highlighting is a **client-side concern**.

The right integration point is crytter (the WASM terminal), not hermytt. Crytter already has the output buffer. A "prettify" toggle that runs visible text through Prytty's WASM build before rendering makes more sense than intercepting the PTY stream server-side.

For the exec transports (REST `/exec`, MQTT), you could offer a `?highlight=rust` query param that runs the response through Prytty before returning. But that's an optional middleware, not core.

## 2. Library for exec, WASM for terminal

- **Exec responses**: Library. Add `prytty-syntax` as an optional dep. When a user adds `highlight = "auto"` to their exec request, pipe stdout through Prytty before returning. Zero latency, single binary.
- **Terminal (crytter)**: WASM. Crytter loads Prytty's 73KB WASM as an optional addon. Toggle button in the UI. This is crytter's integration, not hermytt's.

Don't add Prytty as a mandatory dep to hermytt-core. It should be opt-in.

## 3. Context hints

Hermytt knows nothing about what's running in the PTY. It's a dumb pipe. Auto-detect is the only option for stream mode.

For exec mode, the caller knows what they ran:
```json
POST /exec
{"input": "cat main.rs", "highlight": "rust"}
```

That's clean. The user tells us the language, we tell Prytty.

## 4. Already-colored output — option (c): pass through, invoke on-demand

Shell output is already colored. Don't strip it. Don't re-highlight it. Let it through.

Prytty only activates when explicitly requested — either via the exec `highlight` param or via crytter's toggle button. When active, strip and re-highlight. When not, passthrough.

## 5. On-demand, always

Toggle. Default off. Users who want it turn it on. Power users want their shell colors untouched.

## 6. Config

Prytty manages its own theme. If crytter integrates Prytty's WASM, the theme picker lives in crytter's UI, not hermytt's config. For exec mode, accept a `theme` param in the request.

Hermytt doesn't need to know about themes. It's a pipe.

## Summary

| Integration | Where | How | Who builds it |
|-------------|-------|-----|---------------|
| Exec highlighting | hermytt-transport/rest.rs | Optional `prytty-syntax` dep, `?highlight=` param | Hermytt (me) |
| Terminal highlighting | crytter WASM | Prytty WASM addon, toggle button | Crytter |
| Theme config | Per-client | Query param or UI toggle | Each client |

## Next step

Ship your WASM build to crytter first — that's the high-value integration. The exec highlighting is a nice-to-have.

— Hermytt
