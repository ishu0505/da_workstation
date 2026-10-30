# DA Workstation: hackathon planning handoff

Planning baseline: October 28, 2026. All dates and daily targets use Asia/Singapore.
This file records the user's agreed direction and proposed implementation details.
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

## 3. Dates and work boundaries

- Wednesday October 28: planning and this handoff only.
- Thursday October 29: planning, UX decisions, contracts, acceptance checklist, and manual agent-kit setup if invoked. No coding or experiments.
- Friday October 30: no implementation scheduled. Optional planning only.
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
