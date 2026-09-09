+++
base = "catalogue"
prompt = 'Write a class `Page` in `src/Page.php` of this project, in the namespace `Catalogue`. It takes the rows of the catalogue and the number of rows of one page, and it gives one page of rows by the number of that page. Write a test of it in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-quality"

[[graders]]
type = "file_exists"
path = "src/Page.php"

[[graders]]
type = "file_content"
path = "src/Page.php"
pattern = '(private|protected|public|int|string|array|float|bool) \$[A-Za-z_][A-Za-z0-9_]*\s*=\s*'
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Page.php"
pattern = '\?(int|string|float|bool|array|[A-Z])'
match = "not_contains"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that each parameter of the new class is required, that it wrote no default value in a signature and no nullable type, and that it ran the tests after that change.
