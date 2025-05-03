from apscheduler.schedulers.blocking import BlockingScheduler
from tooka.core import load_tasks

def start_scheduler():
    scheduler = BlockingScheduler()
    tasks = load_tasks()

    for task in tasks:
        scheduler.add_job(task["func"], "interval", seconds=task["interval"], id=task["name"])

    print("🚀 Tooka is running. Press Ctrl+C to exit.")
    scheduler.start()
