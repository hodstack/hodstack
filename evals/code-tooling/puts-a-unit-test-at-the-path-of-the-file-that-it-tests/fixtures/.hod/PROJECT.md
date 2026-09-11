# The intention of this project

This project is a catalogue. It reads rows over HTTP and gives them a title. It is an application, and no different developer installs it.

Run `composer install` to install the dependencies. Run `composer test` to run the tests. Run `php -S 127.0.0.1:8000 -t public` to start the program.

The directory `src/` holds the code. The directory `tests/` holds the tests in `tests/Unit/` and `tests/Feature/`, one file for one case, and a case gives back `true` when it holds. The directory `packages/` holds every version of every dependency of this project.

This project reaches no network. Read `packages/` for the release notes of a package. Do not fetch a page and do not call an interface over HTTP.
