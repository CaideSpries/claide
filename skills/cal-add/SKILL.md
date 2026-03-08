---
name: cal-add
description: Add a new event to Google Calendar.
metadata: {"claide":{"emoji":"📅","requires":{"bins":["gcalcli"]}}}
---

# Cal Add

Create a new Google Calendar event.

## Usage

1. Extract: title, date, start time, end time (or duration)
2. Default timezone: Africa/Johannesburg
3. If end time not specified, default to 1 hour duration
4. Confirm details before creating

```bash
gcalcli add --title "[title]" --when "[YYYY-MM-DD HH:MM]" --duration "[minutes]" --noprompt
```

Confirm: "Event created: [title] on [date] at [time]"
