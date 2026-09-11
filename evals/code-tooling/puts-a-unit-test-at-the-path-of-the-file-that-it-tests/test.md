+++
base = "catalogue"
prompt = 'Write a class `Rows` in `src/Support/Rows.php` of this project, in the namespace `Catalogue\Support`. It takes the rows of the catalogue and it gives the count of them. Write a test of it. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-tooling"

[[graders]]
type = "file_exists"
path = "src/Support/Rows.php"

[[graders]]
type = "file_exists"
path = "tests/Unit/Support/RowsTest.php"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that it wrote the test of `src/Support/Rows.php` at `tests/Unit/Support/RowsTest.php`, at the path of the file that it tests under the directory of the unit tests. It reports that it ran `composer test` and that the tests passed.
