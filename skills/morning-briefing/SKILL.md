---
name: morning-briefing
description: Generate the daily morning briefing with weather, calendar, birthdays, and newsletter highlights.
metadata: {"claide":{"emoji":"🌅","requires":{"bins":["morning-briefing"]},"always":false}}
---

# Morning Briefing

Deliver a daily summary covering weather, calendar, birthdays, and news.

## Trigger

Runs via cron at 05:00 SAST on weekdays (Mon-Fri). Can also be triggered manually.

## Script

```bash
workspace/scripts/morning-briefing
```

## Output Format

```
Good morning, Caide.

**Weather** — [current conditions, high/low]

**Today's Calendar**
- [HH:MM] [Event title]
- ...

**Birthdays**
- [Name] — [date] (if any upcoming)

**Newsletter Highlights**
- [Summary from check-email]
```

If a section has no data, omit it rather than showing "none".
