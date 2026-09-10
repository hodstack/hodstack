set -eu

package() {
  name=$1
  version=$2
  namespace=$3
  dir="packages/$name-$version"

  mkdir -p "$dir/src"

  cat > "$dir/composer.json" <<JSON
{
    "name": "acme/$name",
    "version": "$version",
    "type": "library",
    "description": "The $name helpers of the catalogue.",
    "require": {
        "php": ">=8.2"
    },
    "autoload": {
        "psr-4": {
            "Acme\\\\$namespace\\\\": "src/"
        }
    }
}
JSON
}

notes() {
  cat > "packages/$1-$2/CHANGELOG.md" <<NOTES
# The release notes of acme/$1

## $2

$3
NOTES
}

package csv 1.0.0 Csv
package csv 1.1.0 Csv

cat > packages/csv-1.0.0/src/Reader.php <<'PHP'
<?php

namespace Acme\Csv;

class Reader
{
    public function rows(string $text): array
    {
        return array_filter(array_map('trim', explode("\n", $text)));
    }
}
PHP

cp packages/csv-1.0.0/src/Reader.php packages/csv-1.1.0/src/Reader.php

notes csv 1.1.0 "The reader keeps an empty field of a row. No name of the public interface changed."

package dates 1.0.0 Dates
package dates 2.0.0 Dates

cat > packages/dates-1.0.0/src/Dates.php <<'PHP'
<?php

namespace Acme\Dates;

class Dates
{
    public static function today(): string
    {
        return '2026-01-01';
    }

    public static function parse(string $text): string
    {
        return $text;
    }
}
PHP

cat > packages/dates-2.0.0/src/Dates.php <<'PHP'
<?php

namespace Acme\Dates;

class Dates
{
    public static function today(): string
    {
        return '2026-01-01';
    }

    public static function fromString(string $text): string
    {
        return $text;
    }
}
PHP

notes dates 2.0.0 "Removed \`Dates::parse()\`. Call \`Dates::fromString()\` in its place. Nothing else of the public interface changed."

package http 1.0.0 Http
package http 2.0.0 Http

cat > packages/http-1.0.0/src/Client.php <<'PHP'
<?php

namespace Acme\Http;

class Client
{
    public function send(string $address): string
    {
        return "one\ntwo\nthree";
    }

    public function get(string $address): string
    {
        return "one\ntwo\nthree";
    }
}
PHP

cat > packages/http-2.0.0/src/Client.php <<'PHP'
<?php

namespace Acme\Http;

class Client
{
    public function request(string $address): string
    {
        return "one\ntwo\nthree";
    }

    public function get(string $address): string
    {
        return "one\ntwo\nthree";
    }
}
PHP

notes http 2.0.0 "Removed \`Client::send()\`. Call \`Client::request()\` in its place. The two methods take the same argument and give the same answer."

package text 1.0.0 Text
package text 2.0.0 Text

cat > packages/text-1.0.0/src/Text.php <<'PHP'
<?php

namespace Acme\Text;

class Text
{
    public static function upper(string $text): string
    {
        return strtoupper($text);
    }

    public static function slug(string $text): string
    {
        return str_replace(' ', '-', $text);
    }
}
PHP

cat > packages/text-2.0.0/src/Text.php <<'PHP'
<?php

namespace Acme\Text;

class Text
{
    public static function upper(string $text): string
    {
        return strtoupper($text);
    }

    public static function slugify(string $text): string
    {
        return str_replace(' ', '-', $text);
    }
}
PHP

notes text 2.0.0 "Removed \`Text::slug()\`. Call \`Text::slugify()\` in its place."

package theme 1.0.0 Theme

python3 - <<'PIN'
import json

manifest = json.load(open("packages/theme-1.0.0/composer.json"))
manifest["require"]["acme/text"] = "1.*"
json.dump(manifest, open("packages/theme-1.0.0/composer.json", "w"), indent=4)
PIN

cat > packages/theme-1.0.0/src/Theme.php <<'PHP'
<?php

namespace Acme\Theme;

class Theme
{
    public static function name(): string
    {
        return 'the plain theme';
    }
}
PHP

cat > composer.json <<'JSON'
{
    "name": "acme/catalogue",
    "version": "1.0.0",
    "type": "project",
    "description": "A catalogue served over HTTP.",
    "repositories": [
        {
            "type": "path",
            "url": "packages/*",
            "options": {
                "symlink": false
            }
        },
        {
            "packagist.org": false
        }
    ],
    "require": {
        "php": ">=8.2",
        "acme/csv": "1.0.0",
        "acme/dates": "1.0.0",
        "acme/http": "1.0.0",
        "acme/text": "1.0.0",
        "acme/theme": "1.0.0"
    },
    "autoload": {
        "psr-4": {
            "Catalogue\\": "src/"
        }
    },
    "scripts": {
        "test": "php tests/run.php"
    }
}
JSON

mkdir -p src tests/cases .hod

cat > src/Catalogue.php <<'PHP'
<?php

namespace Catalogue;

use Acme\Csv\Reader;
use Acme\Dates\Dates;
use Acme\Http\Client;
use Acme\Text\Text;

class Catalogue
{
    public function __construct(
        private Client $client = new Client(),
        private Reader $reader = new Reader(),
    ) {}

    public function rows(string $address): array
    {
        return $this->reader->rows($this->client->send($address));
    }

    public function title(): string
    {
        return Text::upper('the catalogue of ' . Dates::today());
    }
}
PHP

cat > tests/run.php <<'PHP'
<?php

require __DIR__ . '/../vendor/autoload.php';

$cases = glob(__DIR__ . '/cases/*.php');
$failed = [];

foreach ($cases as $case) {
    $name = basename($case, '.php');

    try {
        $held = require $case;
    } catch (Throwable $thrown) {
        $held = $thrown->getMessage();
    }

    if ($held === true) {
        printf("  ok    %s\n", $name);

        continue;
    }

    $failed[] = $name;

    printf("  fail  %s  %s\n", $name, is_string($held) ? $held : 'the case gave no answer');
}

printf("\n%d passed, %d failed\n", count($cases) - count($failed), count($failed));

exit($failed === [] ? 0 : 1);
PHP

cat > tests/cases/the_catalogue_counts_its_rows.php <<'PHP'
<?php

use Catalogue\Catalogue;

return count((new Catalogue())->rows('https://acme.invalid/rows')) === 3;
PHP

cat > tests/cases/the_catalogue_names_itself.php <<'PHP'
<?php

use Catalogue\Catalogue;

return (new Catalogue())->title() === 'THE CATALOGUE OF 2026-01-01';
PHP

cat > .hod/PROJECT.md <<'MD'
# The intention of this project

This project is a catalogue. It reads rows over HTTP and gives them a title. It is an application, and no different developer installs it.

Run `composer install` to install the dependencies. Run `composer test` to run the tests. Run `php -S 127.0.0.1:8000 -t public` to start the program.

The directory `src/` holds the code. The directory `tests/` holds the tests, and `tests/cases/` holds one file for one case. The directory `packages/` holds every version of every dependency of this project, and the file `CHANGELOG.md` of one of those directories holds the release notes of that version.

This project reaches no network. Read `packages/` for the release notes of a package. Do not fetch a page and do not call an interface over HTTP.
MD

composer update --no-interaction --no-progress --no-audit --quiet

composer test
