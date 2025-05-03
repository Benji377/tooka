import typer
from rich import print
from tooka.scheduler import start_scheduler

app = typer.Typer()

@app.command()
def run():
    """Run your Tooka automation tasks"""
    print("[bold magenta]Tooka Scheduler Starting...[/bold magenta]")
    start_scheduler()

@app.command()
def init():
    """Create an example Tookafile"""
    with open("Tookafile.py", "w") as f:
        f.write("""\
from time import sleep

def say_hello():
    print("👋 Hello from Tooka!")

TASKS = [
    {"name": "hello", "interval": 10, "func": say_hello}
]
""")
    print("[green]✅ Created Tookafile.py[/green]")
