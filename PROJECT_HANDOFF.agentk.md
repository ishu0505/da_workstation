# DA Workstation: hackathon planning handoff

Planning baseline: October 28, 2026. All dates and daily targets use Asia/Singapore.
This file records the user's agreed direction and proposed implementation details.
Schedule update, October 30: the user requests a small Rust focus bar that opens apps and terminals today. See D-006 and section 13.
Product requirements and dates below are agreed; technical layout and time boxes are proposals to validate during implementation.

## 1. Read this before agent-kit setup

Project root: `/Users/ishu/Projects/pers_random/da_workstation`.
Kit source: `/Users/ishu/Projects/pers_random/agent_dev_kit`.
This is a personal repository. The kit is not installed at the time of writing.

Use this handoff as input to the kit's bootstrap interview and proposal. Follow the actual BOOTSTRAP.md; do not replace its scripts with improvised versions. Reuse the answers already settled here instead of asking them again. Ask about unresolved choices only when necessary. Present the exact configuration and root map for the bootstrap's required approval before installation.

The user will invoke setup manually. That invocation authorizes kit scaffolding and planning documents only. Implementation, dependency installation, API experiments, and app execution start October 31 unless the user changes the schedule. Reading this plan does not authorize commits, pushes, outbound messages, submission, or automatic future runs.

Recommended kit settings, subject to the setup proposal:

- New personal project; own maps; share mode `agent` (agent files shared, learning docs local).
- Use `.agentk.md` for kit planning, decisions, experiment records, and teaching material.
- Record Codex as used; ask whether Claude Code or other development agents also need wiring. Future product providers are separate from the agents used to develop this repository.
- Keep one app initially. Add unit maps when actual folders meet kit thresholds.
- Use an experiment ledger because early integration results decide implementation choices.
- Priorities: working tiling/focus/delegation flow, reliable task state, clear UX, cost, expansion.
- Include a short root-map pointer: read this handoff when changing hackathon scope, implementation milestones, or provider boundaries. Keep detailed specifications out of AGENTS.md.
- Create the kit's PLAN.agentk.md as a concise work tracker pointing here for detailed requirements. Keep architectural decisions and evidence in the kit's decision records; record later scope changes there and update this baseline when explicitly requested.
- Preserve the user's graphify rule: when `/graphify` is invoked, read the installed graphify skill before acting.
- Do not invent build/test commands in maps. Add commands after the scaffold exists and they have been checked.

## 2. Product and required outcome

A native developer workstation that combines window focus/tiling with persistent task chats. The user delegates coding work or experiments to Muse, keeps working elsewhere, receives useful updates, and returns to a task without losing context.

The required November 2 build has three working capabilities: native tiling, a focus/command bar, and real Muse task delegation. A mock result or screenshot-only prototype does not satisfy these requirements.

Each task has a sidebar tab and conversation. It holds the goal, project folder, selected provider, session reference, progress, logs, approvals, artifacts, and next action. The user can take over or give another instruction in the same task.

Future vision: tasks from Slack mentions, personal reminders/to-dos, configurable follow-ups, several agent providers at once, routing rules, richer workspace profiles, and broader OS/app automation.

Planning clarification, October 30: the workstation must also help the user manage each coding agent's skills, MCP connections, plugins, and tools. See section 6 for scope and permission alerts. These additions do not change the build dates or the Muse-first boundary.

## 3. Dates and work boundaries

- Wednesday October 28: planning and this handoff only.
- Thursday October 29: planning, UX decisions, contracts, acceptance checklist, and manual agent-kit setup if invoked. No coding or experiments.
- Friday October 30: build the first app launcher on the current Fedora machine (D-006). Other implementation dates remain unchanged.
- October 30 continuation: the user approves existing-window focus next, followed by the summon/tiling/Muse sequence. D-007 records the focus increment.
- Saturday October 31: implementation day 1.
- Sunday November 1: implementation day 2; complete the integrated demo and freeze features.
- Monday November 2: implementation day 3; finish verification, fixes, packaging, and demo material.
- Tuesday November 3: submit after the user reviews the final package and authorizes submission.

The organizer's public page checked October 28 lists November 4 at 9 PM Eastern, equivalent to November 5 at 10 AM Singapore. November 3 is the user's chosen submission day. Recheck portal requirements and deadline before submission; never schedule work against the last possible hour.

## 4. Scope and cut order

### Required for November 2

1. macOS app with sidebar task tabs and a summonable focus/command bar.
2. Native master/stack tiling of supported windows, with a clear focused target.
3. At least one real coding or experiment task delegated to Muse in a chosen project directory.
4. Running, blocked/needs-input, failed, cancelled, and completed task states.
5. Progress/results in the task tab; meaningful alerts; a follow-up instruction on the same task.
6. Local persistence for task conversations and execution metadata; honest recovery after restart.
7. A context card that explains the goal, recent changes, current outcome, and next action.
8. A provider boundary with one implemented Muse adapter.

### Add only after the required flow works

- Manual to-do creation, due times, snooze, and scheduled follow-up while the app runs.
- Paste a Slack message or link and classify it as an actionable request or FYI, preserving its source.
- Live Slack mentions from an explicitly connected workspace with required permissions.
- A second simultaneous Muse task if isolation and monitoring are already reliable.

### After November 2

- Codex and Claude subscription-backed adapters, subject to supported interfaces and account entitlements.
- Automatic routing, fallback rules, richer provider handoffs, and mixed-provider concurrent tasks.
- Linux/GNOME integration, complete window-manager behavior, multiple displays, and arbitrary app drivers.
- SAM/vision fallback, universal browser automation, subscription-agent multiplexing, and local-model routing.
- Reminders while the app is closed, unless a separately installed background service is deliberately implemented.

Cut extras in this order if time is tight: visual effects, live Slack, scheduled follow-ups, multiple concurrent runs, advanced layouts. Preserve tiling, the focus bar, and real Muse delegation. If one of those is blocked, report it immediately and propose a concrete alternative for the user's decision.

## 5. Primary demo and acceptance criteria

Use a small real local project and bounded experiment, with a reproducible failure fixture available for the alert demo. Label fixtures and replay data clearly.

1. Launch the workstation and create an experiment task linked to that project.
2. Use the focus bar to select a native app. Tile two or three supported normal windows into a master area and stack inside the available screen rectangle.
3. Delegate a bounded comparison or code change to Muse. Observe real session events and output in the task tab.
4. Change focus while Muse runs. Session activity must not type into an unrelated human app or steal the physical pointer.
5. Trigger or encounter a failure. Show source evidence, the explanation, and an actionable next step. Do not turn a proposed diagnosis into an established fact.
6. Respond in the task chat; continue or retry with preserved history. Show the final artifact and verification result.
7. Restart the workstation. Restore task history and mark an unverified old run interrupted/unknown until reconciled with the actual session/process.

Acceptance gates:

- Tiling: supported resizable windows stay within the calculated work area and do not overlap each other or the visible command/sidebar area. Handle absent windows and permission denial without crashing. Document full-screen and minimum-size restrictions.
- Focus bar: a usable registered hotkey summons it; selecting a task opens its tab; selecting an app focuses it; Escape dismisses it. Choose a shortcut containing an actual key after checking conflicts.
- Muse: one task actually executes; the user can inspect output, answer an approval request, stop work, and send a follow-up. Completion reports evidence and exit/session outcome.
- Alerts: notify once per meaningful transition, avoid token-by-token notifications, and provide a jump to the relevant task.
- Persistence: preserve conversation, task status, provider/session reference, and artifact paths. Distinguish an agent's final message from successful verification.
- Budget: measure usage where supplied; disclose unavailable cost data instead of reporting zero.

## 6. Task model and interaction

Proposed states: Inbox, Ready, Running, NeedsYou, Done, Failed, Cancelled, Snoozed. Store execution attempts separately so retries do not erase earlier evidence.

A task stores its ID, title, goal, source/manual-or-Slack reference, project directory, provider ID, timestamps, due/follow-up time if used, and current state. Each run stores session ID, attempt ID, process information, event offset, outcome, usage, and artifact references. Conversations and events are append-oriented and ordered per run.

Examples of user intent, not a mandatory command grammar:

- Create a task to compare two configurations in this project.
- Ask Muse to run it and alert me on completion or failure.
- Focus my editor and put the terminal in the stack.
- Continue this task with the smaller batch size.
- Remind me to review these results tomorrow, if reminders ship.

Keep the initial command surface small: focus/select, create task, delegate, stop, and continue. A text entry plus explicit controls is acceptable; a full DSL and frecency engine are later work.

For follow-ups, distinguish what the agent can verify (a run produced an artifact or a command passed) from a human task that needs confirmation. An unanswered reminder is still open. Slack messages are input data, not authority to execute commands or send replies.

### Permission requests and sidebar status

Example: task A needs permission to run a command. Task B continues its own work. Task A shows "Needs permission" in its sidebar tab.

A session is one agent conversation with its running work. Each permission request must belong to one task, session, and run attempt. An approval allows a specific action. A normal chat reply does not count as approval.

The sidebar must show which task needs attention without opening each chat. Show a short reason beside the status. Use text and an icon with each color. Never use color alone.

Proposed colors need contrast checks before implementation:

| Status | Proposed color | Meaning |
|---|---|---|
| Running | Blue | The agent continues work. |
| Needs permission | Amber | The agent waits for approval of a specific action. |
| Needs input | Amber | The agent needs an answer or another user action. |
| Failed | Red | The run ended with a failure. |
| Completed | Green | The run ended. Show verification results separately. |
| Cancelled or unknown | Gray | The run stopped, or its current state is unverified. |

"Needs permission" and "Needs input" explain the existing NeedsYou task state. A request for permission is not a failed run.

Required permission flow:

1. Receive the permission request from the agent connection.
2. Link the request to its task, session, and run attempt.
3. Highlight the matching sidebar tab.
4. Send one alert for the new request.
5. Open the matching request when the user selects the alert or tab.
6. Show the requested action, target, reason, and available approval choices.
7. Send the user's choice to the matching agent session.
8. Update the status from the agent's response.

Keep the request visible until the agent resolves it. Show the outcome if the user denies permission. Do not claim that work resumed without evidence.

When several agents run, keep their requests separate. Show how many tasks need attention. A blocked task must not pause unrelated tasks. Never send one task's approval to another task or a newer run.

Before the feature freeze, verify this flow with a real Muse permission request. Verify that the alert opens the correct task. Check approval, denial, and repeated events. Check that repeated events do not create repeated alerts. When concurrent runs ship, also test two pending requests in different tasks. Concurrent execution remains outside the required first-build scope.

### Agent skills, MCP connections, plugins, and tools

A skill gives an agent instructions for a type of work. Model Context Protocol (MCP) connects an agent to tools and data. A plugin adds packaged features or connections. A tool performs an action, such as reading a file or running a command.

The user wants one place to inspect and manage these capabilities for coding agents. Keep each agent's settings separate. Show which capabilities the selected task can use. Show connection errors and permission needs in the task chat.

The first build must show capability limits honestly. It must surface permission requests that the Muse connection supports. Do not promise tool controls that Muse does not expose.

Full capability management follows the required flow. Proposed controls include viewing available capabilities, changing enabled capabilities, and managing connections. Installation, removal, and account connection methods remain open. Verify each agent's supported controls before choosing an implementation. Record which settings apply to a task and which apply to all tasks for that agent.

Keep these controls behind the provider boundary. A provider is the service or agent that executes a task. Preserve each task's provider choice. Support for several agents at once remains a future goal.

## 7. Architecture and provider compatibility

Start with a small Rust 2024 application using GPUI and SQLite. Pin a verified GPUI version/revision during the first build. Use the system event loop for UI work and a compatible async bridge for agent I/O. Validate concrete dependencies on October 31.

Proposed logical modules, not pre-created crates:

- UI: sidebar, focus bar, conversation, context card, task status and approvals.
- Task service: lifecycle, run attempts, event stream and notification policy.
- Muse adapter: session transport, approvals, cancellation, usage and output normalization.
- Platform service: window enumeration, focus, geometry, accessibility permissions and tiling.
- Store: tasks, messages, execution events, settings and layout snapshots.
- Scheduler/input adapters: reminders and Slack only when the required path is stable.

Use typed in-process calls/events first. A separate daemon, microservices, browser wrapper, or workflow engine needs a demonstrated requirement.

Provider contract should express capabilities, start, continue/resume if supported, receive events, respond to approvals, and cancel. Normalize events such as started, output, progress, approval-needed, artifact-produced, usage, failed, and completed. Retain provider-specific payloads outside core task logic where needed.

Tasks select a provider explicitly or inherit the default. Implement Muse only before November 2. Design selection and session ownership so tasks can later use different adapters concurrently without changing the task schema. Automatic routing is future work.

Cross-provider switching uses a portable handoff: goal, concise conversation summary, relevant files, changes made, results, and pending decisions. Do not promise that provider-internal reasoning or native sessions can move between vendors.

Subscription access and API billing are different capabilities. ChatGPT Pro and Claude Pro must use officially supported tooling and their actual account limits when those adapters are built. Do not assume either subscription includes API credits. Verify their integration requirements at that time.

## 8. Day-by-day implementation checklist

### October 31: prove all three required technical paths

Morning: inspect local Muse capabilities/authentication without printing credentials; verify Xcode/Metal prerequisites; build a pinned GPUI minimal shell. Limit framework setup investigation to roughly 90 minutes before reporting a blocker and options.

Midday: implement window enumeration/focus and master/stack geometry for one display. Request Accessibility permission through the normal user flow. Demonstrate two supported apps moving to the calculated rectangles. Keep previous geometry for restoration where feasible.

Afternoon: integrate one Muse session using a documented structured protocol if available. Verify the installed version's actual schema and commands. If necessary, use a documented headless/PTY path while accurately representing its limitations. Implement task start, output and terminal outcome; distinguish workstation-spawned sessions from unrelated terminals.

Evening gate: a native window opens, focus/tiling works on the target apps, and a real Muse task produces visible output. Save a short evidence note for each. Escalate missing requirements before expanding scope.

### November 1: integrate the product

Morning: connect task sidebar, conversations, provider/session ownership and SQLite persistence. Keep blocking I/O off the UI thread.

Midday: add meaningful state transitions, approvals, cancellation, failure/completion alerts, and context cards. Continue the same task with a follow-up. Check cancellation against actual child processes and session semantics.

Afternoon: run the whole demo. Add minimal manual tasks/reminders only after it passes. Explore pasted Slack task capture next; live Slack only if access, permissions, and time permit.

Evening gate: required demo passes end to end. Freeze features and list remaining defects by impact.

### November 2: finish

Morning: fix defects in the required flow. Verify fresh launch, permission denial, missing Muse, bad authentication, network failure, task failure, cancellation, and app restart. Verify window layout against actual supported apps.

Afternoon: package a runnable build with precise setup instructions, permission requirements, supported scope and known limitations. Rehearse the same demo three times; investigate any inconsistent result. Collect actual latency/cost observations without inventing a benchmark claim.

Evening gate: working build, verification evidence, final demo recording, project description, architecture summary, limitations and submission checklist are ready. Check the real portal's required fields and video rules before finalizing the package. No new feature development.

### November 3: submit

Recheck the official requirements; verify repository/build/video links as required by the portal. Show the user the exact submission package. Submit only with explicit authorization. Record confirmation or receipt after successful submission.

## 9. Verification and cost

Write focused tests for layout geometry, task-state transitions, persistence/recovery, and provider-event interpretation where they protect real failure modes. Use a real Muse task and native-window manual checks for integration acceptance. Do not describe mocked provider output as a live integration pass.

The user has $150 in credits; provider attribution and actual remaining balance are not verified. Proposed allocation ceilings: $15 experiments, $65 integration, $30 rehearsals, $40 reserve. Treat these as planning allocations, not cost estimates or permission to spend the entire balance automatically. Bound run duration, context/log payloads and agent steps; stop cancelled work and avoid unbounded retries.

Use process/session events for monitoring; call models when reasoning or summarization is needed. Store credentials in supported local secret/config mechanisms, keep them out of git and task history, and redact sensitive output. Report data sent to cloud providers through the configured integration.

## 10. Open choices for planning

- Target demo apps and display geometry; which windows support resize/focus reliably?
- Is macOS the confirmed November 2 demo platform? This is the current proposal based on the available development machine; Linux is deferred.
- Exact summon shortcut and layout ratios; 70/30 is the initial proposal.
- Which installed Muse build, protocol and authentication flow are available?
- Which real coding/experiment project is appropriate for a short, reproducible demo?
- If multiple tasks can edit one repo, should runs be serialized or placed in separate worktrees? Prefer one active editing run per directory initially.
- Can the user's Slack workspace authorize a supported integration? Pasted-message capture remains available as the smaller first step.
- Which coding agents need kit wiring, and should kit checks be added to CI?
- Reminder behavior while the app is closed is outside the initial promise.

Resolve these by discussion during planning or evidence during the build. Do not present proposals as already verified.

## 11. Source references

- Event: https://events.cerebralvalley.ai/e/meta-ai-developer-hackathon
- User-linked details: https://cerebralvalley.ai/e/meta-ai-developer-hackathon/details
- Muse Code integration overview: https://dev.meta.ai/docs/muse-code
- Meta model availability: https://dev.meta.ai/docs/models
- SAM capabilities: https://dev.meta.ai/docs/sam/overview

These pages were consulted October 28. Version-specific integration facts must be checked against installed tools and current official documentation when implementation begins.

## 12. PRD v4.0 vision and coverage, October 30

The user supplied the expanded Master Project Blueprint and PRD v4.0 on October 30.
This section captures its product intent and proposed design. Sections 3–5 still govern dates, required scope, and acceptance.
This blueprint clarification alone did not authorize October 30 implementation. The later focus-bar request authorizes the slice in section 13.

### Execution and AI routing

- Combine deterministic window/app commands with autonomous agent tasks. Use deterministic operations when a supported operation can express the request.
- Keep real native apps on an open tiling canvas. A translucent sidebar, top status strip, and bottom command/chat bar form the proposed hollow frame.
- Run autonomous work in a separate session, tab, or supported workspace without stealing the user's active pointer or keyboard.
- Prefer structured browser or accessibility control. Treat screenshot-based targeting as a proposed fallback that needs separate validation.
- Preserve three routing destinations: local inference, subscription-backed agent sessions, and Muse multimodal/API execution.
- The supplied blueprint reverses the subscription and Muse tier numbers between sections. Use destination names until the numbering is settled.
- Proposed local work includes intent classification, message refinement, bounded log summaries, anomaly triage, and context snapshots.
- Proposed local engines include Ollama, llama.cpp, Candle, and an MLX bridge. Select an engine only after hardware and integration checks.
- Proposed subscription sessions include Claude, Codex, Gemini, and Muse Code through supported persistent interfaces, potentially pseudo-terminals (PTYs).
- Keep all model-specific behavior behind the provider boundary. Muse remains the first implemented provider; automatic routing follows later.

### Command and interaction vision

These are proposed product syntax examples, not commands implemented or verified in this repository:

- `use-w slack @alex`: focus and tile Slack, select a supported recipient, and prepare a message.
- `--improve-local`, `--improve-muse`, and `--improve-gemini`: refine a draft through the chosen destination.
- `ai-task --muse`: delegate a bounded autonomous workflow with progress, approvals, cancellation, and results in its task chat.
- `watch-w terminal --muse` or `--watch`: attach monitoring instructions to a supported process or window.
- `use-w browser yt "rust gpui architecture"`: search through a supported browser driver and present selectable results.
- `ws switch ai-backend`: restore a saved workspace profile and its context.

Show a draft before sending an AI-refined message. Provide explicit approve, edit, and cancel controls.
Bind approval to the displayed task, run, and action. A global Enter key must not approve an unrelated pending request.
The supplied `[E] Edit` and `Ctrl + E` alternatives need one consistent binding.
Modifier-only summon shortcuts remain proposals; section 5 requires a usable shortcut with an actual key and conflict checks.

### Active Context Sentinel and Process Watcher

Example: a training run fails while the user answers messages in another workspace. The workstation alerts the matching task and preserves the previous context.

- Monitor owned PTY output, error output, process exit events, and supported session events before considering window polling.
- Use deterministic filters for known errors and completion signals. Bound model summaries and escalate only relevant evidence.
- Proposed signals include tracebacks, NaN loss, out-of-memory errors, completion, and available progress metrics.
- Preserve a human context snapshot: goal, project, command/run reference, last meaningful state, recent changes, and next action.
- Show a context restore card when the user returns to a task or profile.
- Link each watcher to a task, run, process/session, and workspace. Distinguish completion, failure, interruption, and unverified state.
- Show source evidence and label model diagnoses as suggestions. Review a proposed fix before applying it to the matching target.
- Provide a jump to the alerted task/window. Deduplicate alerts and bound stored log tails.

Required Muse task monitoring and context cards remain in scope. Independent arbitrary-process watchers and automatic profile snapshots extend that scope.
Assess these extensions after the required flow works; do not silently add them to the three-day acceptance gates.

### Visual design and profiles

- Proposed master/stack ratio: 70/30. Profiles save apps, working directories, URLs, layout ratios, and provider defaults.
- Proposed glass surface: `rgba(9, 9, 11, 0.72)`, with blur where supported and opaque readable text.
- Proposed colors: primary `#FAFAFA`, ghost text `#71717A`, command flags `#38BDF8`, Muse tasks `#A855F7`.
- Proposed alert colors: error `#EF4444`, success `#10B981`. Add labels/icons and check contrast on actual backgrounds.
- Proposed 2px borders: cyan for human focus, purple for agent activity, amber/red for attention. Animation remains optional visual polish.
- Preserve per-app command history and frecency as future conveniences. Task chats remain the source for task progress, results, approvals, and follow-ups.
- Proposed keys include Tab/Shift+Tab for selection, Right Arrow/Ctrl+Y for prediction, and Ctrl+Shift+A for alerts.
- Escape behavior must distinguish dismissing UI, cancelling a draft, and stopping a particular running task.

### Platform, libraries, and storage proposals

The blueprint proposes `WindowManagerBackend`, `UiAutomationBackend`, and `ProcessWatcherBackend` platform contracts.
These can guide the platform boundary without requiring a separate daemon or additional crates before there is evidence.

| Area | Candidates to verify when that area is implemented |
|---|---|
| UI, async, persistence | Rust 2024, GPUI, Tokio bridge, SQLite WAL through rusqlite |
| Browser and command surface | Chrome DevTools Protocol through chromiumoxide; nucleo matching; winnow parsing |
| Linux/GNOME | Shell extension, Mutter window APIs, zbus, AT-SPI2, ashpd/PipeWire portals, libei, clipboard integration |
| macOS | AXUIElement, CGWindowList, objc2 bindings, ScreenCaptureKit, supported input and clipboard interfaces |
| Local compute | Metal/Accelerate or supported Linux GPU backends; model choice depends on measured latency and memory |
| Vision | Proposed Muse plus SAM 3.1 integration; verify model availability, outputs, targeting, and input isolation |

Proposed storage includes `workspaces.toml`, command history, a frecency index, an app entity cache, and watcher/context records.
These augment the task/run/message/event model in section 6. A single watcher/history table cannot replace persistent conversations and separate run attempts.
Resolve native platform data directories before adopting the blueprint's Linux-style `~/.config/ai-workstation/` and `~/.local/share/ai-workstation/state.db` paths.
Store secret references and redacted results. Do not follow the blueprint's suggestion to log retrieved credentials or API keys into workspace history.

### Verification gates and roadmap relationship

The v4 milestones describe the full product: hollow frame/DSL, platform tiling, profiles/history, routing, then deep drivers/agents/watchers.
Use sections 4 and 8 for the November 2 delivery sequence. Prove native tiling and real Muse execution before expanding the command surface.

Treat sub-millisecond control, sub-150ms rewrites, zero latency, zero total cost, and universal background input as unverified claims.
Measure local dispatch, end-to-end action completion, inference latency, resource use, and billed usage separately.
Local execution can avoid cloud API charges; do not equate subscription access or local hardware use with unlimited free execution.
Model names, SAM output semantics, platform support, and permissions have not been verified for this v4 proposal.
Record evidence before locking those technical choices. Preserve the wider vision without presenting every candidate as a build commitment.

## 13. First implementation slice, October 30

The user asks to start today with a Rust focus bar that opens apps and terminals.
Build a small native GPUI launcher on the current Fedora GNOME Wayland machine.
Discover installed desktop apps, filter by name or metadata, select with keys, and send a launch request through the desktop integration.
Ghostty is installed; searching for terminal should include it. Show empty results and launch errors clearly.

This slice establishes the Rust build, native UI, text input, keyboard selection, and a platform launch boundary.
It does not prove existing-window focus, global summon, reserved screen edges, tiling, task persistence, or Muse integration.
Keep those as subsequent phases. The final demo platform remains open; a Fedora launcher does not verify macOS support.
Finish November 2 and submit November 3 only after explicit authorization, as before.

## 14. Existing-window focus continuation, October 30

The user accepts the next step: focus an existing app window, or launch the app when none exists.
The current increment uses a small GNOME 50 Shell extension behind `src/windows.rs`.
GNOME denies ordinary callers access to its built-in window-list and focus methods on this machine.
The extension exposes app focus and app window count/focus state through the local desktop session bus.
It does not expose arbitrary script execution. App launch stays in the Rust desktop boundary.

Enter and result clicks request focus first. A confirmed absence of windows permits a launch request.
An unavailable extension produces an error; Ctrl+Enter provides an explicit launch request.
An isolated GNOME test proves launch, existing-window focus, and repeated activation without duplicate windows.
The initial desktop session could not discover the installed extension. On October 30 continuation, enabling succeeds and GNOME reports it active.
The Rust focus command succeeds for Ghostty, with one focused window in subsequent state checks.
The user confirms search typing, selection, and Enter focus Ghostty through the bar without opening another window.
See `gnome-extension/README.agentk.md` and `experiments/gnome-focus/REPORT.agentk.md` for setup and evidence.

The user confirms Linux Ctrl+Alt+Space summons and reopens the bar.
The next tiling increment adds Ctrl+T/Ctrl+R through the GNOME adapter. Isolated two-window placement and restoration pass; desktop and three-window acceptance remain pending.
Real Muse execution remains a subsequent increment.
The November 2 completion date, November 3 submission authorization, and open final demo platform remain unchanged.
