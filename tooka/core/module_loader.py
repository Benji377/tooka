from modules.shell import ShellModule
from modules.file import FileModule

# Add more as needed

MODULE_REGISTRY = {
    "shell": ShellModule,
    "file": FileModule,
}

def load_module(name: str, config: dict):
    cls = MODULE_REGISTRY.get(name)
    if not cls:
        raise ValueError(f"Unknown module: {name}")
    return cls(config)
