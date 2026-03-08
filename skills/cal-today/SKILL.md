---
name: cal-today
description: Show today's calendar events.
metadata: {"claide":{"emoji":"📅","requires":{"bins":["gcalcli"]}}}
---

# Cal Today

Show today's calendar events from Google Calendar.

## Usage

```bash
gcalcli agenda --nostarted --details end "$(date +%Y-%m-%dT00:00:00)" "$(date +%Y-%m-%dT23:59:59)"
```

Present events in a clean format with times.
