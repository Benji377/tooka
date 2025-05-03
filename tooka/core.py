import importlib.util

def load_tasks(filepath="Tookafile.py"):
    spec = importlib.util.spec_from_file_location("Tookafile", filepath)
    tooka_module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(tooka_module)
    return getattr(tooka_module, "TASKS", [])
