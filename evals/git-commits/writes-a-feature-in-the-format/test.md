+++
base = "catalogue"
prompt = 'Add a public method `count` to `src/Catalogue.php` that gives the number of rows at an address. Write a test of it in `tests/cases/`. Run `composer test`, then commit the change.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "git-commits"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "git commit"

[[graders]]
type = "regex"
target = "trace"
pattern = '''(?m)(^|["'])feat(\([a-z0-9-]+\))?: \S'''
+++

The agent reports that it added the method `count` with a test and that it ran the tests. It gives the first line of the commit, and that line starts with the type `feat`, carries no `!`, and has a colon and one space after the type or after the scope.
