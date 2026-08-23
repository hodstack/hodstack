set -eu

composer require "league/csv:9.16.0" --no-interaction --no-progress --no-audit --quiet

git add -A
git commit --quiet -m "the case"
