# AoC CLI
A CLI for running Advent of Code solutions.


## Installation
AoC CLI is currentlt in development and is not yet available to download.

## Quickstart
### Authentication
The CLI requires an AoC session cookie to be set in the environment.
The token can be obtained from the application tab of your browser's developer tools.
To set the token, run the following command and paste the token when prompted:
```shell
aoc token
````

### Trusting a directory
Because the CLI will run all Python files in a directory and subdirectories,
it is important to only run the CLI in a directory you trust.
To assist with this, the CLI will refuse to run in a directory that hasn't
been trusted.
To trust a directory, run the following command,
replacing `<dir>` with the directory you want to trust:
```shell
aoc trust add <dir>
```
> [!CAUTION]
> Only trust directories you trust.
> Trusting malicious directories or directories containing malicious directories
> will result in arbitrary code execution.

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
