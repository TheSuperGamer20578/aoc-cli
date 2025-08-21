> [!IMPORTANT]
> # Advent of Code CLI has moved to GitLab
> The repository can now be accessed at <https://gitlab.com/sgme/aoc-cli/>.

---

<details>
<summary>Original README</summary>

# AoC CLI
A CLI for running Advent of Code solutions.


## Installation
AoC CLI can be installed with Cargo using the following command:
```shell
cargo install --git https://github.com/TheSuperGamer20578/aoc-cli
```

## Quickstart
### Trusting a directory
Because the CLI will run all Python files in a directory and subdirectories,
it is important to only run the CLI in a directory you trust.
To assist with this, the CLI will refuse to run in a directory that hasn't
been trusted.
To trust a directory, run the following command,
replacing `<dir>` with the directory you want to trust:
```shell
aoc trust <dir>
```
> [!WARNING]
> Only trust directories you trust.
> Trusting malicious directories or directories containing malicious directories
> will result in arbitrary code execution.

### Authentication
The CLI requires an AoC session cookie to be set in the environment.
The token can be obtained from the application tab of your browser's developer tools.
To set the token, run the following command and paste the token when prompted:
```shell
aoc token
```

### Writing a solution
When running through the CLI, you have access to the `aoc` module,
which provides the `solution` decorator.
The `solution` decorator requires three arguments:
the year, the day, and the part.
Any functions decorated with the decorator will be called with the input data
as a string, and the return value will be submitted.
The following is an example of a solution file
(of course you'll want to return something other than 0):
```python
from aoc import *

@solution(2023, 1, 1)
def part_one(data: str) -> int:
    return 0

@solution(2023, 1, 2)
def part_two(data: str) -> int:
    return 0
```

### Running solutions
To run solutions, run the following command in a trusted directory:
```shell
aoc run [year] [day] [part]
```
If the year, day, or part is omitted, all solutions for what is provided will be run.

## Templates
Tera templates are supported to quickly generate solution files.
Templates are stored in the `templates` directory under the trusted directory.
The following variables are available:

| Variable | Description                    |
|----------|--------------------------------|
| `year`   | The year of the solution       |
| `day`    | The day of the solution        |
| `part`   | The part of the solution       |
| `url`    | The url to the problem's page  |

The following is an example of a template:
```python
"""{{ url }}"""
from aoc import *


@solution({{ year }}, {{ day }}, 1)
def part1(data: str):
    ...


@solution({{ year }}, {{ day }}, 2)
def part2(data: str):
    ...
```
To render a template, run the following command:
```shell
aoc new <template> <file> [year] [day] [part]
```
This will render the template at `templates/<template>.tera` to `<file>`.
If the year, day or part is omitted, they will render as ellipses (`...`).
The template above would render to the following
with a year of 2023, and a day of 1:
```python
"""https://adventofcode.com/2023/day/1"""
from aoc import *


@solution(2023, 1, 1)
def part1(data: str):
    ...


@solution(2023, 1, 2)
def part2(data: str):
    ...
```

</details>
