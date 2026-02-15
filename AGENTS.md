# AGENTS.md

This file defines repository-specific development rules for Codex agents.

## 0) Core Engineering Principles

- Always add concise comments for key structs and functions (especially cross-module contracts, state transitions, and non-obvious branching).
- Always handle errors explicitly. Never silently swallow errors unless intentionally degraded behavior is documented in code comments.
- Prefer pragmatic architecture: design for current scale and near-term extension, but avoid speculative over-design.

## 1) Architecture And Scope

- Frontend: React + TypeScript + Vite (`src/`).
- Desktop/backend: Tauri 2 + Rust (`src-tauri/`).
- Do not bypass this split. System capabilities must be implemented in Rust and exposed to frontend via Tauri commands.

## 2) State And Data Ownership

- `Snapshot` is the UI source of truth for runtime state.
- Settings mutations must go through backend commands and return latest `Snapshot`.
- Avoid local duplicated state for backend-owned fields.

## 3) Frontend Standards

### 3.1) General

- Keep domain types centralized in `src/types.ts` when shared across components.
- Keep computation helpers in `src/utils/`.
- Use function components + hooks with explicit props types.
- Use `useMemo` for derived data (grouping/filtering/trend building) from snapshot rows.
- Prefer [`uPlot`](https://github.com/leeoniya/uPlot) for chart rendering to keep interaction and rendering performance stable on large datasets.
- Keep naming consistent:
  - Components/files: PascalCase.
  - Variables/functions/types: camelCase/PascalCase by TypeScript convention.
- UI copy can be Chinese; code identifiers should stay English.
- For user-facing copy (titles, descriptions, helper text), prioritize vivid and easy-to-understand language; avoid flat, overly literal, or rigidly technical phrasing.

### 3.2) Chakra UI v3 Conventions (Required)

- Use Chakra UI v3 as the default UI layer for frontend screens.
- Provider setup must use v3 system API:
  - `createSystem(defaultConfig, ...)` in `src/theme.ts`
  - `<ChakraProvider value={system}>` in app entry
- Prefer Chakra primitives/components (`Box`, `Flex`, `Stack`, `Text`, `Button`, etc.) over custom CSS classes.
- Prefer design tokens and semantic tokens over hard-coded values when practical (`bg`, `fg`, `border`, etc.).
- Put app-level global styles in `globalCss` inside the Chakra system config, not ad-hoc scattered CSS.
- For reusable visual patterns, prefer Chakra recipes/slot recipes over duplicated per-component style objects.
- Keep Chakra cascade layers enabled by default unless there is a proven override/conflict need.
- Avoid legacy v2 patterns (`extendTheme`, `ChakraProvider theme=...`, old style-config APIs).
- Prefer `asChild` composition pattern when integrating third-party primitives/components.
- Keep dependencies aligned with v3 migration guidance:
  - Required: `@chakra-ui/react`, `@emotion/react`
  - Do not re-introduce: `@emotion/styled`, `framer-motion` (not required by Chakra v3)
- If a Chakra component typing/API is unstable for current use case, use stable primitives or native element fallback with clear local styling.

### 3.3) Design Token Consistency

- For reusable visual language (glass surface, gradients, bar fills, interaction colors), prefer shared tokens/constants over one-off inline values.
- If a style is used across multiple panels, extract it once (theme token or shared style module) and reuse it.
- New UI changes should preserve visual coherence with existing tokenized styles before introducing a new color family.

## 4) Backend Standards

- Tauri commands live in `src-tauri/src/command.rs`.
- Keep commands focused: one responsibility per command.
- Persist config via `app_config.rs`; runtime collector behavior via `collector.rs`.
- For file persistence, prefer atomic write pattern (temp file + rename).
- When command cannot acquire state lock, return safe fallback snapshot/result instead of panic.

## 5) Privacy And Data Handling

- Keep data local-first. Do not introduce cloud upload/sync by default.
- Preserve secure-input and app exclusion protections.
- Any change that may affect sensitive input handling must be treated as high risk and verified.

## 6) Delivery Workflow

### 6.1) Implementation Order

- Prefer implementing in this order for feature work:
  1. Rust command/state/config changes.
  2. Frontend types update.
  3. UI wiring and interaction.
  4. Validation/build checks.
- Keep frontend/backend contract names aligned (invoke command names and payload fields).

### 6.2) Branch Workflow For New Features (Required)

- When user explicitly asks to "实现新功能/feature" or "do refactor", always execute this Git workflow first:
  1. Checkout `main`.
  2. Pull latest `origin/main`.
  3. Checkout a new branch with `codex/` prefix.
- Before switching branch or implementing feature work, if local changes are present (staged or unstaged), stop and ask user to confirm how to handle them.
- Do not carry unrelated residual changes into the new feature branch without user confirmation.

### 6.3) API Contract Change Discipline

- When changing a Tauri command response shape, update all contract layers in the same change:
  1. Rust payload structs and command comments.
  2. Frontend shared types (`src/types.ts`).
  3. UI wiring/props that consume the payload.
  4. At least one backend test covering the updated response semantics.
- Avoid partial contract migration across multiple PRs unless explicitly requested.

### 6.4) Unrelated Diff Isolation

- Before commit, inspect staged and unstaged changes and exclude unrelated files by default.
- Do not include formatting-only noise or editor/tooling artifacts in feature commits unless explicitly requested by the user.
- If unrelated changes are discovered, either split them into separate commits or leave them out of the feature PR.

### 6.5) Local Experiment Area

- `_lab/` is a local experiment sandbox directory for temporary feature spikes.
- Treat `_lab/` as out-of-scope for normal feature work unless the user explicitly asks to read or modify it.
- Do not include `_lab/` changes in commits/PRs unless the user explicitly requests it.

### 6.6) Commit Style

- Follow conventional-style commits with scope when possible:
  - `feat(settings): ...`
  - `fix(command): ...`
  - `refactor(config): ...`
  - `ci: ...`

### 6.7) PR Body Formatting Hygiene

- When creating or editing GitHub PR descriptions, do not submit escaped newline literals like `\\n` in body text.
- Always use real multiline markdown for PR bodies; prefer `gh pr create/edit --body-file <file>` when content has headings/lists.
- After updating a PR body, verify rendered/raw content once to ensure formatting is correct.

## 7) Quality Gates

- TypeScript must pass strict compile checks (`npm run build` includes `tsc`).
- Rust tests should pass in `src-tauri` (`cargo test`).
- Tauri app build should remain valid (`npm run tauri build`).
- For meaningful feature changes, run at least:
  - `npm run build`
  - `cargo test --manifest-path src-tauri/Cargo.toml`

## 8) Scope Boundaries (Non-Goals)

- Do not introduce heavy state-management libraries.
- Do not split into monorepo/workspace tooling.
- Do not change storage format compatibility casually (must preserve existing local data readability).
