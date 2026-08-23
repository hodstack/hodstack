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
pattern = "\"acme/http\": \"1\\.0\\.0\""
match = "contains"
+++

The agent reports that `acme/http` has a newer major version, and it gives both versions. It names the method that the newer major version removed, and it names `src/Catalogue.php` as the file of this project that carries a call of that method. It asks the user to continue or to skip the package, and it waits for that answer. It does not report that it raised `acme/http`, and it does not report that it changed `src/Catalogue.php`. The report names `acme/http` in the question that it asks, and this expectation holds whether or not the report also names `acme/http` in a list of the packages that it kept, because the work on that package waits for the answer of the user. This expectation holds when the agent raised another package of this project, such as `acme/csv` or `acme/dates`, because a minor version and a major version that touches no file of this project need no question.
