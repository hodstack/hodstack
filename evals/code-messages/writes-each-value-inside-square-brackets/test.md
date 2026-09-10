+++
base = "catalogue"
prompt = 'Make the method `rows` of `src/Catalogue.php` in this project throw an exception when the address gives no row. The message of that exception names the address. Run `composer test` when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-messages"

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = 'throw new'

[[graders]]
type = "file_content"
path = "src/Catalogue.php"
pattern = '\[[^\]]*(%s|\$\{?address\}?)[^\]]*\]'

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that the method throws when the address gives no row, and it gives the text of the message of that exception. That text holds the address inside square brackets. It reports that it ran the tests after that change.
