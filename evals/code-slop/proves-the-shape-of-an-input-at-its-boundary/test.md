+++
base = "catalogue"
prompt = 'Add two methods to `src/Catalogue.php` in this project. The method `rowsOf` takes the text of a JSON document that holds a list under the key `rows`, where each row holds the keys `title` and `price`. The method `report` gives one line for each of those rows, with the title and the price of it. Write a test of each method in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-slop"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'json_decode'

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'json_decode[\s\S]*json_decode'
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'function report\(string'
match = "not_contains"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'throw new'

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that one function reads the document, and it names that function. It reports that the method `report` takes the rows that this function gives, thus no check of a type stands after that place. It reports that the document that holds no rows stops the program. It reports that it ran the tests after that change.
