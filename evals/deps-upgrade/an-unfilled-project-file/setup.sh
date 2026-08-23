set -eu

cat > .hod/PROJECT.md <<'TEMPLATE'
# The intention of this project

Write the intention of the project here. Say what the project does and who uses it.

Give the command that installs the dependencies, the command that runs the tests and the command that starts the program. Name each directory and give its function.

Write a decision that one directory needs in the `AGENTS.md` file of that directory.
TEMPLATE

git add -A
git commit --quiet -m "the case"
