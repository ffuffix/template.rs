---
paths:
  - "**/*.rs"
---

# Comments

Default to no comment. A comment must tell the reader something the code cannot. Anything else is noise that goes stale.

Write one only for:

- A constraint that is not visible from the code: `// The API rejects batches over 100 items, so chunk before sending.`
- A workaround, with its reason and what would let it be removed.
- An algorithm or invariant that a careful reader would still get wrong.

Never write:

- A restatement of the next line, the name, or the type: `// create the client`, `/// Returns the name.`
- Narration of control flow: `// now loop over the items`.
- History, change notes, or a justification of your edit: `// switched to trim()`, `// fixed: handles empty input now`. That is you talking to the reviewer, and it belongs in the commit message.
- An explanation of a convention that `.claude/CLAUDE.md`, a rule, or `.github/CONTRIBUTING.md` already documents.
- Commented-out code.

## Where comments belong

- `///` doc comments document `pub` items for callers. A private item gets one only when its name cannot carry the meaning.
- An explanation that covers a whole file goes in one header at the top (`//!`), not scattered through the body.
- `// SAFETY:` comments and the `reason` on `#[expect]` are required by lints. Make them state the actual justification.

When a change makes a comment wrong, fix or delete it in the same edit.
