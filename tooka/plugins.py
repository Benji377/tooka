import pluggy

hookspec = pluggy.HookspecMarker("tooka")
hookimpl = pluggy.HookimplMarker("tooka")

class TookaSpec:
    @hookspec
    def register_tasks():
        """Return a list of additional tasks."""

def load_plugin_tasks():
    pm = pluggy.PluginManager("tooka")
    pm.add_hookspecs(TookaSpec)

    try:
        from plugins import example_plugin
        pm.register(example_plugin)
    except ImportError:
        pass

    results = pm.hook.register_tasks()
    return [task for sublist in results for task in sublist]
