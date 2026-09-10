+++
base = "catalogue"
prompt = 'Add a method `count` to `src/Catalogue.php` in this project that gives the number of rows of an address. Write a test of it in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-slop"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = '(interface|abstract|trait) '
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'function count\(string \$[A-Za-z]+\): int'

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = '//|/\*'
match = "not_contains"

[[graders]]
type = "git_clean"
paths = ["composer.json", "tests/cases/the_catalogue_names_itself.php"]

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that it wrote one method and that it wrote no interface, no abstraction, no option and no parameter that the task does not need. It reports that it wrote no comment. It reports that it ran the tests after that change.
