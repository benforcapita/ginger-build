# Ginger Code — Progress Tracker

> עדכון אחרון: 2026-08-14 · Commit `10b3d5b` (main)

## איפה אנחנו

Ginger Code הוא אפליקציית desktop workspace (Tauri 2 + Rust + React/TS). בנינו את כל **15 ה-slices של LLD Part I** (הליבה: editor, terminal, git, agents, diff/review/apply, verification, packages, presence, recovery, packaging) — ואז כתבנו את **LLD Part II** (240 סעיפים) והתחלנו לממש אותו.

**הסטטוס הנוכחי:** כל ה-backend services של LLD Part II נכתבו (40+ מודולי Rust), חוברו ל-`lib.rs`, וה-commit נדחף ל-GitHub. **הקוד לא הודק/הורץ עדיין** — רק נכתב.

---

## מה כבר גמור ✅

### LLD Part I — 15 slices (commits `b61b892` → `9b8c69e`)
| Slice | תוכן | Commit |
|---|---|---|
| 1 | Desktop foundation + Action Registry | `b61b892` |
| 2 | Neovim editor host | `1762d05` |
| 3 | Workspace persistence + SQLite (13 tables) | `5bc07fe` |
| 4 | PTY terminal host | `0fa92b3` |
| 5 | Git service | `431d444` |
| 6 | Agent supervisor (worktrees, max 3) | `df1c0e0` |
| 7 | Agent Dock UI + Command Palette | `8cb98b8` |
| 8 | Diff/review/apply pipeline | `c94d24e` |
| 9 | Verification service | `9f31eb5` |
| 10 | Package manager + curated catalog | `88cfc97` |
| 11 | Project detection + recommendations | `e08362e` |
| 12 | Ginger presence system | `51dade5` |
| 13 | Crash recovery + safe mode | `6f4e172` |
| 14 | macOS packaging + update validation | `d4fb70d` |
| 15 | E2E stabilization + test definitions | `9b8c69e` |

### LLD Part II — כתיבה ראשונית (commit `10b3d5b`)
- **`docs/LLD-Part-II.md`** — מסמך מלא, 240 סעיפים (commit `c51fd22`)
- **40+ מודולי Rust** נכתבו לדיסק, כל אחד ממפה סעיפים ספציפיים:
  - `process/` (64), `jobs/` (63), `agent_adapter/` (65-66), `trust/` (85), `ginger_config/` (86), `settings/` (87-88), `search/` (74), `state_machine/` (135-142), `verification_runs/` (95-96), `command_detect/` (96), `review/` (98-99), `apply/` (100-101), `reconcile/` (132-134), `diagnostics/` (125-127), `ipc/` (130), `integrity/` (124), `ollama/` (70-72), `package_plan/` (81-82), `cache/` (84), `keybinding/` (90), `task/` (93-94), `scheduler/` (159-160), `cleanup/` (102), `environment/` (83), `watcher/` (150), `terminal_state/` (146-147), `recommend/` (79-80), `supply_chain/` (123), `flags/` (165), `compat/` (169), `correlation/` (220), `progress/` (221-222), `atomic/` (225), `locking/` (224), `concurrency/` (223), `path/` (219), `time/` (218), `serialization/` (217), `error/` (215)
- **`types.rs`** (214) — type-safe ID wrappers
- **`events.rs`** (214) — typed domain events
- **`src/types/domain.ts`** (216) — TS domain contracts
- **`lib.rs`** — כל 4 העריכות גמורות: mod declarations, use statements, service registration ב-`.setup()`, וכל ה-commands ב-`generate_handler!`

---

## מה נשאר לעשות ביחד 🔜

### שלב 1 — לוודא שהקוד עובר קומפילציה (קריטי)
הקוד נכתב בלי להריץ. לפני כל המשך, צריך:
- [ ] `cargo check` / `cargo build` ב-`src-tauri/` — לתקן שגיאות קומפילציה
- [ ] לוודא ש-`chrono` ו-`sha2` (ועוד deps) נמצאים ב-`Cargo.toml` — יש סיכוי שחסרים
- [ ] `pnpm build` / `tsc` בצד ה-frontend
- [ ] לתקן את זהות ה-git (`Ben Blum <benblum@macbookpro.local>` → email אמיתי)

### שלב 2 — להשלים את ה-slices החסרים של LLD Part II
חלק מהסעיפים נכתבו כמודולים, אבל **לא כולם מחוברים ל-UI/commands מלאים**. ה-slices המתוכננים:
- [ ] **Slice 16** — Process Supervisor + Background Jobs + Domain Events (רובו כתוב, צריך לוודא חיבור)
- [ ] **Slice 17** — Agent Adapter Discovery + Custom Agents + Startup Prompt Strategy
- [ ] **Slice 18** — Workspace Trust + `.ginger` config + Settings layers
- [ ] **Slice 19** — Search (ripgrep) + LSP/Formatter management
- [ ] **Slice 20** — State Machines (workspace/agent/task/review/verification/recovery)
- [ ] **Slice 21** — Golden Paths + Milestones A-F + Diagnostics/Health

### שלב 3 — סעיפים שעדיין לא מומשו (פערים)
סעיפים שכתובים ב-LLD אבל **אין להם מודול/מימוש עדיין**:
- [ ] **73** Project Explorer (UI)
- [ ] **75** Symbols (מ-LSP)
- [ ] **76-78** LSP / Formatter / Treesitter management
- [ ] **91-92** Command Palette providers (files/symbols/agents/tasks/worktrees/packages/settings/recent)
- [ ] **103-104** macOS File Permissions + Notifications
- [ ] **105-108** App Menu, Deep Links, Single Instance, CLI launcher (`ginger .`)
- [ ] **109-111** First-Run Validation + Onboarding
- [ ] **112-113** Ginger Welcome Screen + Empty Agent Dock art
- [ ] **114-116** State Arbitration + Commentary
- [ ] **117-119** Accessibility + Theme + Font
- [ ] **120** Telemetry (disabled by default)
- [ ] **121-122** Update channels + Catalog updates
- [ ] **128-129** Action Inspector + Event Inspector
- [ ] **131** DB migration testing
- [ ] **143-145** Performance budgets + Large repo strategy
- [ ] **148-151** Clipboard, Drag&Drop, File watcher, External changes
- [ ] **152-155** Agent attribution + Commit workflow + Branch switching
- [ ] **156-160** Agent worktree update + Concurrency + Queue (חלקית)
- [ ] **166-168** Build config + CI + Release pipeline
- [ ] **171-178** Test fixtures + Test strategies
- [ ] **183-192** Toast/Status bar/Agent dock + Search actions + Neovim compat
- [ ] **204-212** Definition of Done + Milestones A-F
- [ ] **233-237** Golden Paths

### שלב 4 — בדיקות (הושארו לסוף, לפי בקשתך)
- [ ] Rust unit tests
- [ ] TS tests
- [ ] Integration tests
- [ ] E2E tests (A–E, כבר הוגדרו ב-slice 15)

---

## Milestones A-F (יעד עליון)
- **A** — Ginger boots and edits
- **B** — Isolated fake agent
- **C** — Real terminal agents
- **D** — Managed developer environment
- **E** — Recovery-grade application
- **F** — Release candidate

---

## הערות
- **סגנון עבודה:** כותבים קבצים אחד-אחד, בלי להריץ; בדיקות בסוף.
- **המודל:** `ollama/deepseek-v4-flash:cloud`
- **Repo:** `github.com/benforcapita/ginger-build` · branch `main`
- **הצעד הבא המומלץ:** `cargo check` כדי לוודא שהקוד שנכתב עומד — לפני שממשיכים לכתוב עוד.
