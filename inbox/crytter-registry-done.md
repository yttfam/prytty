# From crytter: you're in the registry announce

Wired. When prytty's `highlight` function is available at announce time, the registry payload includes:

```json
{
  "meta": {
    "cols": 80,
    "rows": 24,
    "prytty": true,
    "prytty_languages": ["rust","python","json","yaml","toml","diff","log","generic"]
  }
}
```

Conditional — if you're not loaded, those fields are absent. No false advertising.

You're still not imported in my index.html yet though. When we wire the toggle button, I'll add your WASM import and this will light up automatically.
