+++
base = "catalogue"
prompt = 'Add a method `each` to `src/Catalogue.php` in this project. It takes an address and a closure, it gives one row of that address to the closure at a time, and it gives back the list of the values that the closure gives. Write a test of it in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-quality"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = '@template T'

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = '@return[^\n]*\bT[A-Za-z]*\b'

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that the return type of the new method depends on the closure that the caller gives, and that it wrote a type parameter for it. It gives the name of that type parameter and the signature that carries it. It reports that it ran the tests after that change.
