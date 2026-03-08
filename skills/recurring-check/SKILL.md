---
name: recurring-check
description: List all active recurring reminders.
metadata: {"claide":{"emoji":"🔁","requires":{}}}
---

# Recurring Check

Read `workspace/data/recurring.txt` and list all active recurring reminders.

## Output Format

```
Active recurring reminders:
1. [frequency] at [time] — [message] (via [channel])
```

If none: "No recurring reminders set."
