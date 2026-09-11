# The intention of this project

This project is a catalogue. It reads rows over HTTP and gives them a title. It is a package, and a different developer installs it with `composer require acme/catalogue`.

The class `Catalogue\Catalogue` in `src/Catalogue.php` is the public API of this package. Each other type of `src/` stands behind that boundary, and no caller outside this package calls it.

Run `composer install` to install the dependencies. Run `composer test` to run the tests.

The directory `src/` holds the code. The directory `tests/` holds the tests, and `tests/cases/` holds one file for one case. The directory `packages/` holds every version of every dependency of this project.

This project reaches no network. Read `packages/` for the release notes of a package. Do not fetch a page and do not call an interface over HTTP.
