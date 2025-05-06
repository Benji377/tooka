from .base import BaseModule
import subprocess


class ShellModule(BaseModule):
    def run(self) -> str:
        """
        Executes the shell command defined in the config and returns the output.
        """
        # Get the command from the config passed when the module is instantiated
        command = self.config.get("command")

        if not command:
            raise ValueError("No shell command provided in the config.")

        # Run the shell command
        result = subprocess.run(command, shell=True, capture_output=True, text=True)

        if result.returncode != 0:
            # In case of failure, we can return stderr to the caller
            return f"Error: {result.stderr}"

        # Return stdout as the result of the module run
        return result.stdout
