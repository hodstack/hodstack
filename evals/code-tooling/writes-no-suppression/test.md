+++
base = "catalogue"
prompt = 'Add a method `first` to `src/Rows.php` in this project that gives the first row. Write a test of it in `tests/cases/`, and run `composer analyse` and `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-tooling"

[[graders]]
type = "file_content"
path = "src/Rows.php"
pattern = '@analyse-ignore'
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Rows.php"
pattern = 'function count\(\): int'

[[graders]]
type = "file_content"
path = "src/Rows.php"
pattern = 'function first'

[[graders]]
type = "file_content"
path = "analyse.json"
pattern = '"level": 9'

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer analyse"
+++

The agent reports that `src/Rows.php` held a suppression, that it deleted that suppression, and that it corrected the code that gave the fault. It names the return type that it wrote. It reports that it kept the level in `analyse.json` and that `composer analyse` and `composer test` pass.
