---
name: cal-week
description: Show this week's calendar events.
metadata: {"claide":{"emoji":"📅","requires":{"bins":["gcalcli"]}}}
---

# Cal Week

Show calendar events for the next 7 days.

## Usage

```bash
gcalcli agenda --nostarted --details end "$(date +%Y-%m-%dT00:00:00)" "$(date -d '+7 days' +%Y-%m-%dT23:59:59)"
```

Present events grouped by day.
