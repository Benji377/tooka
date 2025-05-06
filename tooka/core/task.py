"""Contains the Task class"""

from .module_loader import load_module

class Task:
    """Represents a single task that can be scheduled"""
    def __init__(self, task_data: dict):
        """
        Initializes a Task object with all its attributes given as a dictionary
        """
        self.name = task_data["name"]
        self.schedule = task_data["schedule"]
        self.output = task_data.get("output", None)
        self.module_defs = task_data["modules"]
        self.modules = self._load_modules()

    def _load_modules(self):
        """
        Loads a required module from the modules directory
        """
        instances = []
        for module_def in self.module_defs:
            for name, config in module_def.items():
                instances.append(load_module(name, config))
        return instances

    def run(self):
        """
        Runs the task by running each module synchronously and saving their output
        """
        results = []
        for module in self.modules:
            result = module.run()
            results.append(result)
        self._write_output(results)

    def _write_output(self, results):
        """Specifies where and how the output is written"""
        if not self.output:
            return
        with open(self.output, "a", encoding="utf-8") as f:
            for line in results:
                f.write(line + "\n")
