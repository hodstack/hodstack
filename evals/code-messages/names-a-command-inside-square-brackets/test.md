+++
base = "catalogue"
prompt = 'Write a class `Address` in `src/Address.php` of this project, in the namespace `Catalogue`, that reads the address of the catalogue from the variable `CATALOGUE_ADDRESS` of the environment. It throws when that variable holds nothing, and the message of that exception names the variable and tells the reader to run the command `composer catalogue:setup`. Write a test of it in `tests/cases/`. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-messages"

[[graders]]
type = "file_exists"
path = "src/Address.php"

[[graders]]
type = "file_content"
path = "src/Address.php"
pattern = '\[CATALOGUE_ADDRESS\]'

[[graders]]
type = "file_content"
path = "src/Address.php"
pattern = '\[composer catalogue:setup\]'

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that the class throws when the variable of the environment holds nothing, and it gives the text of the message of that exception. That text holds the name of the variable inside square brackets and the command inside square brackets. It reports that it ran the tests after that change.
