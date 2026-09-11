+++
base = "catalogue"
prompt = 'Add a class `Rows` in `src/Rows.php` in this project. It takes the text of the rows of an address, it gives the count of them, and the method `rows` of `src/Catalogue.php` is the one caller of it. Write a test of it in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-quality"

[[graders]]
type = "file_content"
path = "src/Rows.php"
pattern = '@internal'

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = '@internal'
match = "not_contains"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that `.hod/PROJECT.md` says that this project is a package, and that `src/Catalogue.php` is its public API. It reports that it marked the new class `@internal`, because no caller outside this package calls it, and that it left `src/Catalogue.php` without that marker. It reports that it ran the tests after that change.
