import threading
import time

_tools = []


def append(tool):
    _tools.append(tool)


def _clear():
    for tool in _tools:
        tool.stop()
    _tools.clear()


def _run_thread():
    while True:
        if not threading.main_thread().is_alive():
            break
        time.sleep(1)

    _clear()


def unregister():
    _clear()


_thread = threading.Thread(
    target=_run_thread
)
_thread.start()
