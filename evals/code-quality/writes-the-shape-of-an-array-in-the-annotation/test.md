+++
base = "catalogue"
prompt = 'Add a method `lengths` to `src/Catalogue.php` in this project. It takes an address and it gives the length of each row of that address, keyed by the text of the row. Write a test of it in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-quality"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'function lengths\(string \$[a-zA-Z]+\): array'

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = '@return\s+(non-empty-)?array<string,\s*([a-z-]+-)?int>'

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that the return type `array` of the new method cannot hold the shape of the array, and that it wrote that shape in the annotation of the static analyser above the signature. It gives that annotation and the signature that carries it. It reports that it ran the tests after that change.
