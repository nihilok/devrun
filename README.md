# run

A simple scripting language for CLI automation. Define functions in a `Runfile` (or `~/.runfile`) and call them from the command line to streamline your development workflow.

## Installation

Clone this repo and build with Cargo:

```sh
cargo install devrun
```

## Usage

- Run a script file:
  ```sh
  run myscript.run
  ```
- Call a function defined in your `Runfile`:
  ```sh
  run build
  run test
  run lint
  ```
- Pass arguments to functions:
  ```sh
  run start dev
  run deploy production
  ```
- Start an interactive shell (REPL):
  ```sh
  run
  ```

## Runfile Examples (npm, uv, docker, arguments)

```runfile
# Example for python, node, and docker
python:install() uv venv && uv pip install -r requirements.txt
python:test() uv pip install pytest && pytest
node:dev() npm install && npm run dev
node:lint() npm run lint
docker:build() docker build -t myapp .
docker:run() docker run -it --rm myapp
docker:shell() docker compose exec $1 bash
docker:logs() docker compose logs -f $1
git:commit() git commit -m "$1" && echo "${2:-Done}"
echo_all() echo "$@"
```

### Calling Nested Functions and Passing Arguments

- To call a nested function, use space-separated names:
  ```sh
  ./run python test
  ./run docker shell app
  ./run docker logs web
  ./run git commit "Initial commit" "That's done!"
  ./run echo_all hello world!
  ```
- Arguments are passed positionally and available as `$1`, `$2`, `$@`, etc. Default values can be set using shell syntax (e.g., `${2:-done}`).

## Configuration

- Place your `Runfile` in the project root or in your home directory as `.runfile`.
- Functions are defined as `name()` followed by a command on the same line.
- Arguments can be passed to functions and accessed as `$1`, `$2`, `$@`, etc.
- Defaults are also supported (as in bash) `${1:-default}`

## License

MIT
