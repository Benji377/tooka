"""Contains the file module"""

import os
import subprocess
from subprocess import CalledProcessError
from .base import BaseModule


class FileModule(BaseModule):
    """A module to interact with files on the system"""
    def __init__(self, config: dict):
        """
        Calls the parent module and then does some simple checks to the given file
        """
        # Make sure to call the parent class constructor
        super().__init__(config)
        self.file_path = self.config.get("file")
        self.command = self.config.get("on-change")

        if not self.file_path or not self.command:
            raise ValueError("Both 'file' and 'on-change' must be provided in the config.")

        # Store the last modified time of the file
        self.last_modified = os.path.getmtime(self.file_path)

    def run(self) -> str:
        """
        Checks if the file has changed since the last run.
        If the file has changed, executes the 'on-change' command.
        """
        try:
            # Get the current modification time of the file
            current_modified = os.path.getmtime(self.file_path)

            # Check if the file has changed
            if current_modified != self.last_modified:
                # Update the last_modified timestamp
                self.last_modified = current_modified

                # Run the command defined in the config
                result = subprocess.run(self.command, shell=True, capture_output=True, text=True, check=True)

                if result.returncode != 0:
                    # In case of error, return stderr
                    return f"Error: {result.stderr}"

                # Return stdout as the result of the file change action
                return result.stdout
            return "No changes detected."
        except CalledProcessError as e:
            return f"Error: {str(e)}"
