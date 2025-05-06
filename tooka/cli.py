"""The main entry point of the app. Represents all commands of the CLI"""

from typing import Annotated
import importlib.metadata
from rich.console import Console
import typer

app = typer.Typer()
console = Console()

@app.command()
def run(
        name: Annotated[str, typer.Option(help="Name a task to run")] = "all",
        once: Annotated[bool, typer.Option(help="Only run the task once")] = False,
):
    """
    Run your Tooka automation tasks.
    If a task name is specified that differs from 'all', we only run that one task, else run all.
    If the option once is specified, then only run once.
    """
    console.print(f"Called 'run' with: name={name} and once={once}")


@app.command()
def list(
        name: Annotated[str, typer.Option(help="Filter tasks by name pattern")] = "",
        active: Annotated[bool, typer.Option(help="Only show active tasks")] = False,
):
    """
    List all tasks currently running. You can further filter by name patterns (simple regex) and state.
    Returns a table of tasks with name, interval, state and source file like so:
    ```
    {
         "name": "ping",
         "interval": "30s",
         "enabled": true,
         "source": "Tookafile"
    },
    ```
    """
    console.print(f"Called 'list' with: name={name} and active={active}")


@app.command()
def add(
        name: Annotated[str, typer.Option(help="Name a task to add")],
        interval: Annotated[int, typer.Option(help="Interval in seconds")],
        command: Annotated[str, typer.Option(help="Python import path, e.g. my_module:my_function")],
):
    """
    Add a task to your Tooka automation tasks. This requires a name, an interval to run the task and the python
    function to run.
    The name must be unique! (For the remove function to work)
    Returns true or false (with error) depending on whether the task was added.
    """
    console.print(f"Called 'add' with: name={name} and interval={interval} and command={command}")


@app.command()
def remove(
        name: Annotated[str, typer.Argument(help="Name a task to remove")]
):
    """
    Removes a task by its name. Returns true or false (with error) depending on whether the task was removed.
    """
    console.print(f"Called 'remove' with: name={name}")


@app.command()
def init(
        overwrite: Annotated[bool, typer.Option(help="Overwrite existing file")] = False,
):
    """
    Creates a new example file in the current directory that can be used to create a new task.
    The overwrite option can be used to overwrite existing files.
    Returns true or false (with error) depending on whether the file was created.
    """
    console.print(f"Called 'init' with: overwrite={overwrite}")


@app.command()
def status():
    """
    Returns basic information about Tooka and the state of the tasks
    It outputs stats like uptime, running tasks, next running tasks, etc.
    """
    console.print("Called 'status'")


@app.command()
def validate(
        file: Annotated[str, typer.Argument(help="File to validate, must be a Tooka task file")],
        verbose: Annotated[bool, typer.Option(help="Verbose output")] = False,
):
    """
    Validates a Tooka task file. This should be performed before submitting the task.
    Tasks are validated automatically on submission (see add() command)
    Returns true or false (with error) depending on whether the file was validated.
    If verbose is on, additionally prints extracted task info from file
    """
    console.print(f"Called 'validate' with: file={file} and verbose={verbose}")


@app.command()
def version():
    """
    Simply prints the Tooka version, can be used to check if Tooka is installed.
    """
    app_version = importlib.metadata.version("tooka")
    console.print(f"Tooka v{app_version}")
