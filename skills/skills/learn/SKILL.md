---
name: learn
description: "Write one rule for this project in `.hod/rules/`. Use when the user says that a decision was wrong, corrects your work, gives an instruction for the next task, or asks you to remember something about this project."
disable-model-invocation: true
---

# Learn

Write one fault that happened as one rule in `.hod/rules/`.

## 1. Test the fault

A rule needs a fault that happened. Name the decision that you made wrong, or name the decision that the files of this project refused to give you. A wrong change that you imagine is not a fault.

Stop when one of these statements is true. Say which statement, then write no file.

- An agent finds the fact when it reads the code. Put the fact in the code instead, because a test stops the agent that breaks it and a rule does not.
- The fact belongs to one task. Use it in that task, then write nothing.
- A file in `.hod/rules/` holds this rule. Correct that file instead, and delete the sentence that the correction makes wrong.

## 2. Write the file

Give the file the path `.hod/rules/<name>.md`. Give `<name>` two or three words with a hyphen between them, and name the subject of the rule, such as `queue-worker-restart`.

```markdown
---
name: queue-worker-restart
description: Restart the queue worker after a change to a job class.
---

Run `php artisan queue:restart` after you change a file in `app/Jobs/`. The worker holds the old class in memory, thus the test passes and the job fails.
```

Write the `description` for the agent that reads the index in `AGENTS.md`: it names the subject and the condition, in one sentence, in the imperative.

Write the body in [ASD-STE100](https://www.asd-ste100.org) Simplified Technical English: one instruction in one sentence, the imperative, the active voice, and no synonym for variety. Give the exact path, the exact command and the exact name. Give the reason only when the reason changes the next decision of the agent.

Write one rule in one file. Two faults are two files.

## 3. Ask the user

Show the path and the text of the file. Ask the user to accept it.

Keep the number of the rules: name a rule file that this rule replaces, and delete that file in the same change. Ask the user to accept one more rule when no file goes away.

A correction from the user needs no question. Write it, then say that you wrote it.

## 4. Write the index

Run `hod update --project`. The command reads each file in `.hod/rules/` and writes the index in `AGENTS.md` again.

Report the path of the file and the rule that went away.
