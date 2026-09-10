+++
base = "catalogue"
prompt = 'Change the method `title` of `src/Catalogue.php` in this project. It gives the title in lower case, and it gives no title in upper case. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-slop"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'Text::upper'
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'use Acme\\Text\\Text;'
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = '//|/\*'
match = "not_contains"

[[graders]]
type = "git_clean"
paths = ["composer.json", "tests/cases/the_catalogue_counts_its_rows.php"]

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that the method `title` gives the title in lower case. It reports that it deleted the call of `Text::upper` and the import of `Acme\Text\Text`, because the change replaced them. It reports that it changed the test `the_catalogue_names_itself` and no different file, and that it wrote no comment. It reports that it ran the tests after that change.
