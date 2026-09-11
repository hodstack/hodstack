+++
base = "catalogue"
prompt = 'Add a method `titleOf` to `src/Catalogue.php` in this project. It takes the text of a JSON document that holds the keys `address` and `title`, and it gives the title of that document. Write a test of it in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-quality"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'mixed'
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'function titleOf\(string \$[A-Za-z]+\): string'

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = '\(string\)|\(array\)|\(object\)'
match = "not_contains"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that the new method takes a string and gives a string, and that no signature of it carries `mixed` or an untyped array. It reports that it wrote no cast and no assertion of a type. It reports that it ran the tests after that change.
