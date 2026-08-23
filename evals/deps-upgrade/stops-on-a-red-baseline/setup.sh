set -eu

cat > tests/cases/the_catalogue_counts_every_row.php <<'PHP'
<?php

use Catalogue\Catalogue;

return count((new Catalogue())->rows('https://acme.invalid/rows')) === 5;
PHP

git add -A
git commit --quiet -m "the case"
