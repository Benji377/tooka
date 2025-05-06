from .module_loader import load_module

class Task:
    def __init__(self, task_data: dict):
        self.name = task_data["name"]
        self.schedule = task_data["schedule"]
        self.output = task_data.get("output", None)
        self.module_defs = task_data["modules"]
        self.modules = self._load_modules()

    def _load_modules(self):
        instances = []
        for module_def in self.module_defs:
            for name, config in module_def.items():
                instances.append(load_module(name, config))
        return instances

    def run(self):
        results = []
        for module in self.modules:
            result = module.run()
            results.append(result)
        self._write_output(results)

    def _write_output(self, results):
        if not self.output:
            return
        with open(self.output, "a") as f:
            for line in results:
                f.write(line + "\n")
