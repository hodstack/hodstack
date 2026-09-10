+++
base = "catalogue"
prompt = 'Let a caller of `src/Catalogue.php` in this project use a different reader than the reader that the catalogue holds. Write a test of that behaviour in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-immutability"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'function set'
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = '\$this->reader\s*='
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'new (self|static|Catalogue)\('

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that it wrote no setter and that no method of `src/Catalogue.php` writes a property of its own object. It reports that the new method returns a new catalogue that holds the different reader, and it gives the name of that method. It reports that it ran the tests after that change.
