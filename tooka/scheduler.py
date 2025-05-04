from apscheduler.schedulers.blocking import BlockingScheduler
from tooka.task.task_list import TaskList


class Scheduler:
    tasks_list = TaskList([])
    scheduler = BlockingScheduler()

    def __init__(self):
        self.tasks_list.load_tasks()

    def start_scheduler(self):
        for task in self.tasks_list.get_tasks():
            self.scheduler.add_job(task["func"], "interval", seconds=task["interval"], id=task["name"])

        print("🚀 Tooka is running. Press Ctrl+C to exit.")
        self.scheduler.start()

    def stop_scheduler(self):
        self.scheduler.shutdown()

