"""Template module, use it to create your own"""

class BaseModule:
    """Module base class, all modules need to derive from this class"""
    def __init__(self, config: dict):
        """Initializes the config for the module"""
        self.config = config

    def run(self) -> str:
        """Runs the command or module operation"""
        raise NotImplementedError
