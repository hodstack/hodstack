+++
base = "catalogue"
prompt = 'The method `rows` of `src/Catalogue.php` gives a row two times when the response holds that row two times. Correct it, thus each row comes one time. Run `composer test`, then commit the change.'
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
pattern = '''(?m)(^|["'])fix(\([a-z0-9-]+\))?: \S'''
+++

The agent reports that it corrected the method `rows` and that it ran the tests. It gives the first line of the commit, and that line starts with the type `fix`, carries no `!`, and has a colon and one space after the type or after the scope.
