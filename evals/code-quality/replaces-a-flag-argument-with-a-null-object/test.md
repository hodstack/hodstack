+++
base = "catalogue"
prompt = 'Add a cache to `src/Catalogue.php` in this project: the method `rows` keeps the rows of an address after the first read, and gives them back without a second call to the client. Let a caller turn the cache off. Write a test of each behaviour in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-quality"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'bool \$'
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = '= (true|false)'
match = "not_contains"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that no signature of `src/Catalogue.php` carries a parameter of type `bool`. It reports that it wrote a second implementation of the cache that answers each read with a miss, that the constructor of `Catalogue` takes the cache, and that a caller turns the cache off with that implementation. It reports that it ran the tests after that change.
