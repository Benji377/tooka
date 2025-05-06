"""Contains the Scheduler class, responsible for running tasks on a given time interval"""

from apscheduler.schedulers.background import BackgroundScheduler
from tooka.core.task import Task
from tooka._utils import parse_cron_expression

class TaskScheduler:
    """The Scheduler runs a given task on a given schedule"""
    def __init__(self):
        """
        Initializes the scheduler using the ApScheduler and starting it
        TODO: Tasks should be feed by the TaskManager
        """
        self.scheduler = BackgroundScheduler()
        self.scheduler.start()
        self.tasks = {}

    def add_task(self, task: Task):
        """
        Adds a task to the scheduler
        """
        self.tasks[task.name] = task
        self.scheduler.add_job(
            func=task.run,
            trigger="cron",
            id=task.name,
            **parse_cron_expression(task.schedule)
        )
