set -eu

composer create-project laravel/laravel . \
  --no-interaction --no-progress --quiet --remove-vcs

mkdir -p .hod

cat > .hod/PROJECT.md <<'MD'
# The intention of this project

This project is a Laravel application. It serves a small catalogue over HTTP.

Run `composer install` to install the dependencies. Run `php artisan test` to run the tests. Run `php artisan serve` to start the program.

The directory `app/` holds the code. The directory `tests/` holds the tests. The directory `routes/` holds the addresses that the program answers.
MD

php artisan test
