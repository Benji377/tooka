import json
import os
from .task import Task

class TaskManager:
    def __init__(self, tasks_dir: str):
        self.tasks_dir = tasks_dir
        os.makedirs(self.tasks_dir, exist_ok=True)
        self.tasks = {}

    def load_tasks(self):
        self.tasks = {}  # Clear current tasks
        for filename in os.listdir(self.tasks_dir):
            if filename.endswith(".json"):
                with open(os.path.join(self.tasks_dir, filename), "r") as f:
                    data = json.load(f)
                    task = Task(data)
                    self.tasks[task.name] = task

    def save_task(self, task: Task):
        path = os.path.join(self.tasks_dir, f"{task.name}.json")
        data = {
            "name": task.name,
            "schedule": task.schedule,
            "output": task.output,
            "modules": task.module_defs,
        }
        with open(path, "w") as f:
            json.dump(data, f, indent=4)
        self.tasks[task.name] = task

    def remove_task(self, name: str):
        path = os.path.join(self.tasks_dir, f"{name}.json")
        if os.path.exists(path):
            os.remove(path)
        self.tasks.pop(name, None)

    def get_task(self, name: str):
        return self.tasks.get(name)

    def list_tasks(self):
        return list(self.tasks.values())
