---
name: task-list
description: List all active tasks, optionally filtered by priority or status.
metadata: {"claide":{"emoji":"✅","requires":{}}}
---

# Task List

Show active tasks from the task tracker.

## Usage

List all tasks, or filter:
- "show my tasks" → all active tasks
- "show high priority tasks" → filtered by priority
- "show overdue tasks" → past due date

## Output Format

```
Active tasks:
1. [priority] [description] — due [date]
```

If none: "No active tasks."
