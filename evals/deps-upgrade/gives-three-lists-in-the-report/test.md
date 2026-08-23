+++
base = "catalogue"
intent = "Upgrade the dependencies of this project to newer versions and keep the tests green."

[[graders]]
type = "file_content"
path = "composer.json"
pattern = "\"acme/text\": \"1\\.0\\.0\""
match = "contains"
+++

The report of the agent gives three lists. The first list names each package that the agent raised, with the version before and the version after. The second list names each package that the agent kept and gives a reason for each one, and `acme/text` is one of those packages. The third list names each file of the code that the agent changed, or says that the agent changed no file of the code.
