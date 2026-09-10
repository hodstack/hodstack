+++
base = "catalogue"
prompt = 'Write a class `Row` in `src/Row.php` of this project, in the namespace `Catalogue`. It holds the text of one row of the catalogue and the number of that row, and it gives the text in upper case. Write a test of it in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-immutability"

[[graders]]
type = "file_exists"
path = "src/Row.php"

[[graders]]
type = "file_content"
path = "src/Row.php"
pattern = 'final class Row'

[[graders]]
type = "file_content"
path = "src/Row.php"
pattern = 'readonly'

[[graders]]
type = "file_content"
path = "src/Row.php"
pattern = 'function set'
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Row.php"
pattern = 'protected'
match = "not_contains"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that the new class is closed to extension with `final` and that each property of it is read only. It reports that the class carries no setter. It reports that it ran the tests after that change.
