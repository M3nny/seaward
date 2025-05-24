# flags to be used in the release version
release-flags := "--release"

# default recipe to display help information
default:
	@just --list

# build the project (debug)
build:
	cargo build

# build the project with optimizations
build-release:
	cargo build {{release-flags}}

# run the project with the provided arguments (debug)
run args = "":
	cargo run -- {{args}}

# run the project with optimizations
run-release:
	cargo run {{release-flags}}

# generate the documentation and open it in the browser
doc:
	cargo doc --open
