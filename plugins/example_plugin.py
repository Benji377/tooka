from tooka.plugins import hookimpl

def ping():
    print("🏓 Pong from plugin!")

@hookimpl
def register_tasks():
    return [{"name": "ping", "interval": 30, "func": ping}]
