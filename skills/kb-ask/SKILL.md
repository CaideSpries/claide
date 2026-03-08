---
name: kb-ask
description: Search the Open Notebook knowledge base and answer questions from saved content.
metadata: {"claide":{"emoji":"📚","requires":{"bins":["kb-ask"]}}}
---

# KB Ask

Query the knowledge base to answer questions from previously saved content.

## Usage

```bash
kb-ask "<question>"
```

## Examples

- "What did I save about Rust frameworks?" → `kb-ask "Rust frameworks"`
- "Find that article about pricing strategies" → `kb-ask "pricing strategies"`

Present the answer directly. If nothing relevant found, say so.
