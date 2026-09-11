+++
base = "catalogue"
prompt = 'Add a method `count` to `src/Catalogue.php` in this project that gives the count of the rows of an address. Write a test of it in `tests/cases/`, and run the checks of this project when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-tooling"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer analyse"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"

[[graders]]
type = "file_content"
path = "analyse.json"
pattern = '"level": 9'
+++

The agent names each check that it ran, and it gives the command and the result of each one. Those checks name `composer analyse` and `composer test`.
