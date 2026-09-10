+++
base = "catalogue"
prompt = 'Add a method `sorted` to `src/Catalogue.php` in this project. It takes the rows of the catalogue in a parameter with the name `$rows` and it gives those rows in alphabetical order. Write a test in `tests/cases/` that reads the same rows again after that call. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-immutability"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'sort\(\$rows\)'
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'function sorted'

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that the method `sorted` copies the rows before it sorts them, and that the collection of the caller keeps its order. It reports that it ran the tests after that change.
