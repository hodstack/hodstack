+++
base = "catalogue"
prompt = 'Add a method `expiresAt` to `src/Catalogue.php` in this project. It takes the day that the catalogue was read, as a date object of PHP, and it gives the day seven days after it, as a date object of PHP. Write a test of it in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-immutability"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'DateTimeImmutable'

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'DateTime\b'
match = "not_contains"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that it took the immutable form of the date of PHP, `DateTimeImmutable`, in the place of `DateTime`, in the signature of the new method. It reports that it ran the tests after that change.
