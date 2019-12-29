from shiba import instrumentation
from threading import Thread, main_thread
import time


def _run_thread():
    while True:
        if not main_thread().is_alive():
            break
        time.sleep(1)

    instrumentation.set_exiting()
    _clean()


def _clean():
    with instrumentation.update_state() as state:
        state.library.loaded = False
        state.server.started = False


_thread = Thread(
    target=_run_thread
)
_thread.start()


def unregister():
    _clean()
