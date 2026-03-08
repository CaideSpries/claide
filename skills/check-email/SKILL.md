---
name: check-email
description: Check Boris's Gmail for newsletter highlights and summarize them.
metadata: {"claide":{"emoji":"📧","requires":{"bins":["python3"]}}}
---

# Check Email

Read newsletters from Boris's Gmail inbox and summarize highlights.

## How It Works

1. Run the `read-newsletters` script which:
   - Connects to Gmail via OAuth (token at `workspace/data/gmail-token.json`)
   - Filters by senders in `workspace/data/newsletter-senders.txt`
   - Extracts text from HTML emails
   - Returns summaries within token budget
2. Present highlights in a concise format

## Script

```bash
workspace/scripts/read-newsletters
```

## Output Format

For each newsletter found:
```
**[Sender] — [Subject]**
[2-3 sentence summary of key points]
```

If no newsletters: "No new newsletters since last check."
