from apscheduler.schedulers.background import BackgroundScheduler
from tooka.core.task import Task
from tooka._utils import parse_cron_expression

class TaskScheduler:

    def __init__(self):
        self.scheduler = BackgroundScheduler()
        self.scheduler.start()
        self.tasks = {}

    def add_task(self, task: Task):
        self.tasks[task.name] = task
        self.scheduler.add_job(
            func=task.run,
            trigger="cron",
            id=task.name,
            **parse_cron_expression(task.schedule)
        )
