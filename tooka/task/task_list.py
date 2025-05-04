from typing import List
from .task import Task
import importlib.util


class TaskList:
    def __init__(self, tasks: List[Task]):
        self.tasks = tasks

    def load_tasks(self, filepath="Tookafile.py"):
        spec = importlib.util.spec_from_file_location("Tookafile", filepath)
        tooka_module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(tooka_module)
        return getattr(tooka_module, "TASKS", [])

    def get_tasks(self):
        return self.tasks

    def get_task(self, name) -> Task|None:
        for task in self.tasks:
            if task.name == name:
                return task
        return None

    def add_task(self, task: Task):
        if self.get_task(task.name) is not None:
            raise Exception(f"Task {task.name} already exists")
        self.tasks.append(task)

    def remove_task(self, task: Task):
        if self.get_task(task.name) is None:
            raise Exception(f"Task {task.name} does not exist")
        self.tasks.remove(task)

    def get_task_count(self):
        return len(self.tasks)