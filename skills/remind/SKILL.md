---
name: remind
description: Set a reminder for a specific date and time. Boris will deliver it via the originating channel.
metadata: {"claide":{"emoji":"⏰","requires":{}}}
---

# Remind

Set reminders that fire at a specific time and deliver to the user's channel.

## Usage

When the user asks to be reminded of something:
1. Extract: what to remind about, when (date + time), which channel
2. If time is ambiguous, ask for clarification
3. Default timezone: Africa/Johannesburg (SAST)
4. Store in `workspace/data/reminders.txt`
5. Confirm: "Reminder set: [what] at [when]"

## Format

One reminder per line in `workspace/data/reminders.txt`:
```
YYYY-MM-DD HH:MM | channel | message
```
