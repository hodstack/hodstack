.DEFAULT_GOAL := cli:run

.PHONY: cli\:run cli\:lint cli\:unit cli\:docs cli\:build cli\:msrv cli\:audit cli\:deny cli\:vet cli\:machete cli\:coverage cli\:test skills\:lint typos evals\:test

cli\:run:
	cd cli && cargo run -q -- $(filter-out $@,$(MAKECMDGOALS))

cli\:lint:
	cd cli && cargo make test:lint

cli\:unit:
	cd cli && cargo make test:unit

cli\:docs:
	cd cli && cargo make test:docs

cli\:build:
	cd cli && cargo build --locked --release $(if $(TARGET),--target $(TARGET))

cli\:msrv:
	cd cli && cargo make test:msrv

cli\:audit:
	cd cli && cargo make test:audit

cli\:deny:
	cd cli && cargo make test:deny

cli\:vet:
	cd cli && cargo make test:vet

cli\:machete:
	cd cli && cargo make test:machete

cli\:coverage:
	cd cli && cargo make test:coverage

cli\:test:
	cd cli && cargo make test

skills\:lint:
	npx --yes markdownlint-cli2@0.23.2 --config .markdownlint.json "skills/**/*.md"

typos:
	cd cli && cargo make test:typos

evals\:test:
	cd evals && cargo run -q --release -- $(filter-out $@,$(MAKECMDGOALS)) $(ARGS)

%:
	@:
