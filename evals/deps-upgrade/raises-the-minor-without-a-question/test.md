+++
base = "catalogue"
intent = "Upgrade the dependencies of this project to newer versions and keep the tests green."

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer outdated --direct"

[[graders]]
type = "file_content"
path = "composer.json"
pattern = "\"acme/csv\": \"1\\.0\\.0\""
match = "not_contains"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "git commit"
min = 0
max = 0

[[graders]]
type = "git_dirty"
+++

The agent reports that it raised `acme/csv` from the version that the project held to a newer minor version, and it gives both versions. It reports that it ran the tests after that change. It asked the user no question before it raised that package.
