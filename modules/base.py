class BaseModule:
    def __init__(self, config: dict):
        self.config = config

    def run(self) -> str:
        raise NotImplementedError
