# run

A simple scripting language for CLI automation. Define functions in a `Runfile` (or `~/.runfile`) and call them from the command line to streamline your development workflow.

## Installation

Clone this repo and build with Cargo:

```sh
cargo build --release
```

## Usage

- Run a script file:
  ```sh
  ./run myscript.run
  ```
- Call a function defined in your `Runfile`:
  ```sh
  ./run build
  ./run test
  ./run lint
  ```
- Pass arguments to functions:
  ```sh
  ./run start dev
  ./run deploy production
  ```
- Start an interactive shell (REPL):
  ```sh
  ./run
  ```

## Example Runfile for Node.js

```runfile
build() npm install && npm run build
test() npm test
lint() npm run lint
start() npm run start -- $1
```

## Example Runfile for Python (using `uv`)

```runfile
setup() uv venv && uv pip install -r requirements.txt
test() uv pip install pytest && pytest
lint() uv pip install flake8 && flake8 src/
run() uv python $1
```

## Configuration

- Place your `Runfile` in the project root or in your home directory as `.runfile`.
- Functions are defined as `name()` followed by a command on the same line.
- Arguments can be passed to functions and accessed as `$1`, `$2`, etc.

## License

MIT
