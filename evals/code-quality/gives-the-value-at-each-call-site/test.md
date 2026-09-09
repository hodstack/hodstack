+++
base = "catalogue"
prompt = 'Let a caller of the method `title` of `src/Catalogue.php` in this project choose the words that stand before the date, in place of the words that the method holds today. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-quality"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'function title\([^)]*=[^)]*\)'
match = "not_contains"

[[graders]]
type = "file_content"
path = "tests/cases/the_catalogue_names_itself.php"
pattern = 'title\([^)]+\)'

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that the new parameter is required and that it carries no default value. It names `tests/cases/the_catalogue_names_itself.php` as a file where it gave the value. It reports that it ran the tests after that change.
