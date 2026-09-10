+++
base = "catalogue"
prompt = 'Add a method `fromFile` to `src/Catalogue.php` in this project that reads the rows of the catalogue from the path of a file that the caller gives. Write a test of it in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-slop"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'throw new'

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'return \[\]'
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = '@file|error_reporting|catch \(\\?Throwable'
match = "not_contains"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that the new method throws when the file is absent, and that it gives no empty collection and no fallback value in the place of that fault. It reports that the message of the fault names the path of the file. It reports that it ran the tests after that change.
