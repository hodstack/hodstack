+++
base = "catalogue"
prompt = 'Write a class `Rows` in `src/Rows.php` in this project. It takes the rows of the catalogue and it gives the count of them. Call it from the method `rows` of `src/Catalogue.php`. Write a test of it in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-quality"

[[graders]]
type = "file_content"
path = "src/Rows.php"
pattern = 'declare\(strict_types=1\);'

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that it wrote the line `declare(strict_types=1);` at the top of `src/Rows.php`. It names each parameter and each return type of the new class, and each one carries a type. It reports that it ran the tests after that change.
