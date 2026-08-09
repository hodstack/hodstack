# Hodstack CLI

This directory holds the `hod` program. The program runs the skills. The command `hod <skill> <prompt>` starts a coding agent with that skill and that prompt.

The `AGENTS.md` file at the top level gives the intention of the project and the rules for this file. This file gives the decisions for this directory.

---

## 1. The function of the program

The `skills` directory holds the material. This directory moves the material to the work. The user runs a command such as `hod pr:review 1042` in a terminal, in CI, or in a hook.

The user can start each skill with this program. The Agent Plugins standard has no method to show that a skill is for a user or for a model (refer to `skills/AGENTS.md`, section 3). This program is the start method that the standard does not have.

Write the program in Rust. Compile one binary with the name `hod`. Give the binary no dependence on a runtime on the computer of the user.

---

## 2. Distribution

TBD.
