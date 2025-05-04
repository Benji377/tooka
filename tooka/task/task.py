class Task:
    def __init__(self, name, interval, source):
        self.name = name
        self.interval = interval
        self.source = source

    def __str__(self):
        return f"Task {self.name} runs every {self.interval} seconds with source {self.source}"
