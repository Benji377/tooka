"""Manages tasks by loading them, saving them and parsing them while checking their validity"""

import json
import os
from .task import Task

class TaskManager:
    """Class defining the Task Manager"""
    def __init__(self, tasks_dir: str):
        """
        Initialized the TaskManager with the location of the directory containing all the tasks
        and an empty list where all tasks will be loaded into
        """
        self.tasks_dir = tasks_dir
        os.makedirs(self.tasks_dir, exist_ok=True)
        self.tasks = {}

    def load_tasks(self):
        """
        Loads all the tasks from the tasks directory
        """
        self.tasks = {}  # Clear current tasks
        for filename in os.listdir(self.tasks_dir):
            if filename.endswith(".json"):
                with open(os.path.join(self.tasks_dir, filename), "r", encoding="utf-8") as f:
                    data = json.load(f)
                    task = Task(data)
                    self.tasks[task.name] = task

    def save_task(self, task: Task):
        """
        Writes a new task to the tasks directory
        TODO: Refactor to simply copy the given JSON file
        """
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
        """
        Removes a task from the tasks directory
        """
        path = os.path.join(self.tasks_dir, f"{name}.json")
        if os.path.exists(path):
            os.remove(path)
        self.tasks.pop(name, None)

    def get_task(self, name: str):
        """
        Get a task by its name
        """
        return self.tasks.get(name)

    def list_tasks(self):
        """
        Returns a list of all tasks
        """
        return list(self.tasks.values())
