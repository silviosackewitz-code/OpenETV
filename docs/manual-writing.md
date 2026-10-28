# Help writing conventions

Adapted from Race Analysis `docs/manual-writing.md` and its native manual reader (reviewed 2026-10-27).

- Use **Manual** for the reader, and a contextual **?** with a “Help for …” tooltip.
- One short Markdown page per feature, in English. Use exact visible control labels in bold.
- Task help: ordered steps, the result of each step, and only necessary conditions.
- Operating reference: follow the window; use “Do this | Get that” and “You see | It is” tables when useful.
- Technical reference: keep formulas, file constraints and model limits separate from task instructions.
- Use short natural sentences and familiar words. No filler, history, implementation details or repeated warnings. Give a reason where the user makes the decision.
- A displayed result is not its cause: say what to do next, and name a cause only when known.
- Read each changed page inside the app. Keep wording and pictures aligned with the current UI.

Sources live in `rust/help/`; the native reader embeds them. `rust/MANUAL.md` is the package index. Update both when adding a topic. No browser or Markdown runtime is needed.
