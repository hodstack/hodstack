+++
base = "catalogue"
prompt = 'Add the static analyser `bin/analyse` as a check of this project. Run each check of this project when you finish.'
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "Skill"]

[[graders]]
type = "tool_used"
tool = "Skill"
input_match = "code-tooling"

[[graders]]
type = "file_content"
path = "composer.json"
pattern = '"test":\s*(\[[^\]]*analyse|"[^"\n]*analyse)'

[[graders]]
type = "file_content"
path = "analyse.json"
pattern = '"level": 9'

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that it added `bin/analyse` to the script `test` of `composer.json`, thus `composer test` runs the analyser with the tests. It reports that the level of the analyser in `analyse.json` is 9. It reports that it ran `composer test` and that each check passed.
