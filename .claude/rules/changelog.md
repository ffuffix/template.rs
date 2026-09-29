---
paths:
  - "CHANGELOG.md"
---

# Changelog

Each version is a `###` heading holding only the version number, newest first. Unreleased changes go under `### Unreleased` at the top. Add that heading if it does not exist yet.

Within a version, each category is a top-level bullet and its entries are nested under it:

```markdown
### 0.2.0
- Added
  - `--shout` flag that prints the greeting in upper case.
- Fixed
  - Names wrapped in tabs are now trimmed.
- Removed
  - The `--legacy` flag.
```

- Categories appear in this order: `Added`, `Fixed`, `Removed`, then optional ones such as `Deprecated`, `Changed`, or `Security`.
- Leave out a category that has no entries.
- Write each entry for someone using the crate: what changed for them, in one line. Not a description of the diff.
- Headings carry no date, link, or `v` prefix.
