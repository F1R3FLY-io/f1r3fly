# Contributing

Thank you for contributing to a [F1R3FLY.io](https://github.com/F1R3FLY-io) project. This file is the entry point for new contributors; it pairs with the repo's `README.md` (orientation) and `DEVELOPER.md` (deeper setup) where those exist.

Org-wide policy lives in [F1R3FLY-io/.github](https://github.com/F1R3FLY-io/.github). Anything in this file that contradicts that repo defers to it.

---

## Before You Start

1. Read the repo's `README.md` and, if present, `DEVELOPER.md`.
2. Skim `docs/ToDos.md`, `docs/Backlog.md`, and any open issues — work may already be claimed.
3. Open a **GitHub Discussion or Issue** for non-trivial changes before writing code. F1R3FLY.io follows a **documentation-first** methodology: requirements and design land before implementation.
4. For protocol- or ecosystem-level proposals, file a [FIP](https://github.com/F1R3FLY-io/FIPS) rather than a PR.

---

## Branching and Commits

- Branch from the repo's default branch. Use prefixes: `feature/`, `fix/`, `docs/`, `perf/`, `chore/`.
- Use [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `perf:`, `refactor:`, `test:`, `chore:`.
- Keep one concern per pull request.
- Preserve commit history when picking up someone else's PR — do not squash unrelated commits without consent.
- **Do not** add Claude / Codex / Gemini / other AI co-author trailers, Co-Authored-By lines for AI tools, or "Generated with X" footers. Human Co-Authored-By trailers are welcome.

---

## Local Checks

Run the smallest set relevant to your change, then broader checks before opening the PR.

<!-- BUILD-COMMANDS:BEGIN
     Filled in for this repo. /harmonize will not overwrite this block.
-->
```bash
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test
```
<!-- BUILD-COMMANDS:END -->

If a check is unavailable in your environment, say so explicitly in the PR description rather than skipping silently.

---

## Pull Requests

Every PR should describe:

- **What** changed
- **Why** it changed (link issue / FIP / discussion)
- **How** it was verified (commands run, manual test steps, screenshots for UI)
- **Follow-ups** that remain, if any

A PR is ready for review when:

- [ ] CI is green (or the failure is unrelated and noted)
- [ ] New behavior has tests
- [ ] Public API / CLI / config changes are documented (`README.md`, `docs/`, doc-comments)
- [ ] No secrets, credentials, or PII in code, commits, fixtures, or logs

Maintainers review for correctness, test coverage, scope discipline, and consistency with documented architecture. Expect iteration; respond to feedback in additional commits rather than force-pushing rewrites of reviewed history.

---

## Documentation Expectations

- Update Markdown when commands, ports, flags, paths, or workflows change.
- Examples should be runnable from the repository root unless stated otherwise.
- Prefer `.yaml` for hierarchical/structured data in docs; avoid decorative emoji in synthesized documentation.
- Significant design changes should land an ADR under `docs/architecture/decisions/` or update the relevant spec under `docs/specifications/`.

---

## Stigmergic Collaboration

F1R3FLY.io repositories coordinate through shared markdown files (`docs/ToDos.md`, `docs/work-logs/`, `docs/discoveries/`) so multiple contributors — human and agentic — can work in parallel without stepping on each other.

When you start non-trivial work:

1. Claim the task in `docs/ToDos.md` by setting its YAML frontmatter `status: in_progress` and `claimed_by: <identifier>`.
2. Create a work log at `docs/work-logs/task-{id}-{timestamp}.md` and update it as you go.
3. If you pause or block, leave `handoff_status:` and `next_steps:` so the next contributor can pick up.

**`claimed_by` identifier formats:**

- Human: `human-<your-git-email>` (e.g. `human-jane@example.com`)
- Solo agent: `<tool>-session[-<id>]` (e.g. `claude-session-a1b2c3`, `codex-session`, `gemini-session`)
- Team member: `<team>/<role>` (e.g. `design-sprint/researcher`)

Full conventions: see this repo's `CLAUDE.md` / `AGENTS.md` / `GEMINI.md`, or the org-wide copies in [F1R3FLY-io/.github](https://github.com/F1R3FLY-io/.github).

---

## AI-Assisted Contributions

AI coding assistants (Claude Code, Codex, Gemini, Cursor, Copilot, others) are welcome. We aim to keep guidance **vendor-neutral**:

- Per-tool instructions live in the repo's `CLAUDE.md`, `AGENTS.md`, and `GEMINI.md` files; if only one is present, treat it as the canonical source.
- When committing from an agentic session (the assistant is operating autonomously, not pair-programming), prefix the commit subject with `[agent]`.
- Do **not** add AI attribution footers, Co-Authored-By lines for the assistant, or "Generated with …" trailers. The `[agent]` prefix and the git author email are sufficient signal.
- The contributor is responsible for every line submitted — review your assistant's output the same as you would a human colleague's PR.

---

## Security and Privacy

- Never commit API keys, tokens, credentials, signing keys, or `.env` files. Use environment variables and `.env.example`.
- Strip PII from code, comments, tests, fixtures, logs, and error messages. Use reserved examples: `user@example.com`, `192.0.2.x`, `+1-555-0100`.
- Vulnerabilities: do **not** open a public issue. Email `f1r3fly.ceo@gmail.com` or use GitHub Security Advisories on the affected repo.
- If you accidentally commit a secret or PII: stop, do not push; if already pushed, contact a maintainer immediately — history may need to be rewritten.

---

## License

Unless the repo states otherwise, contributions are licensed under [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). By opening a pull request you agree your contribution may be distributed under that license.

---

## Getting Help

- **Questions / design discussion:** GitHub Discussions on this repo (or the [`.github` org repo](https://github.com/F1R3FLY-io/.github/discussions) if cross-cutting).
- **Bugs:** GitHub Issues with reproduction steps, version/commit, and environment.
- **Process or scope concerns:** mention a maintainer in the relevant issue or PR.

<!-- REPO-SPECIFIC:BEGIN
     Optional. Add anything that's unique to this repo and doesn't belong in
     README.md or DEVELOPER.md — for example: required signing setup, special
     branch policies, or hand-off conventions to downstream repos.
     /harmonize will not overwrite this block.
-->
<!-- REPO-SPECIFIC:END -->
