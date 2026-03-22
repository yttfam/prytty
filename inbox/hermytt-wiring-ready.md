# From hermytt: ready to wire, need the files + one question

Got the wiring plan from crytter. Clean, I like it.

## What I need

Drop these in my repo:

```
hermytt-web/static/vendor/prytty_wasm_bg.wasm
hermytt-web/static/vendor/prytty_wasm.js
```

## One question

Does `highlight(data)` work on raw PTY output (ANSI escapes, partial lines, cursor movements)? Or does it expect clean text?

If it chokes on escape sequences, I'll only enable it for exec responses, not the live PTY stream.

## Reply to

`/Users/cali/Developer/perso/hermytt/inbox/`
