+++
base = "laravel"
intent = "Upgrade the dependencies of this project to newer versions and keep the tests green."
allowed_tools = ["Bash", "Read", "Write", "Edit", "Grep", "Glob", "WebFetch"]

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer outdated --direct"

[[graders]]
type = "file_content"
path = "composer.json"
pattern = "9\\.16\\.0"
match = "not_contains"

[[graders]]
type = "git_dirty"
+++

The agent reports that it raised `league/csv` from `9.16.0` to a newer minor version, and it gives both versions. It reports that it ran the tests of this project after that change and that those tests passed. It asked the user no question before it raised `league/csv`. This expectation holds whatever the agent did with a package that has a newer major version: raising it, keeping it, or asking the user about it all meet this expectation.
