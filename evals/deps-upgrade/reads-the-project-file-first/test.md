+++
base = "catalogue"
intent = "Upgrade the dependencies of this project to newer versions and keep the tests green."

[[graders]]
type = "tool_order"
after = { input_match = "composer outdated" }

[graders.before]
input_match = "(?i)project\\.md"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer test"
+++

The agent reports that it ran the tests of this project after the changes that it made. It names no other command for the tests, such as `vendor/bin/phpunit` or `php artisan test`.
