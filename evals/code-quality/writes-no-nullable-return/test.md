+++
base = "catalogue"
prompt = 'Add a method to `src/Catalogue.php` in this project that finds one row of an address by the text that the row starts with. Write a test of that method in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-quality"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'return null'
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = ':\s*\?'
match = "not_contains"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that the new method returns no null and that it carries no nullable return type. It reports one of two designs: the method throws when it finds no row, or a second method gives a collection. It reports that it ran the tests after that change.
