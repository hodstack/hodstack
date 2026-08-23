+++
base = "catalogue"
intent = "Upgrade the dependencies of this project to newer versions and keep the tests green."

[[graders]]
type = "tool_order"
after = { tool = "Bash", input_match = "composer bump" }

[graders.before]
tool = "Bash"
input_match = "composer require"
+++

The report of the agent names each package that it raised with the version before and the version after, and it names each package that it kept with the reason. It names each file that it changed, and it says that it ran the tests after the last change and that those tests passed. This expectation holds whether or not the report names `composer bump`, because a grader measures that command.
