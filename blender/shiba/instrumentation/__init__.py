from contextlib import contextmanager
from shiba import addon_preferences
from shiba.instrumentation import library, server
from shiba.instrumentation.error import StateMatchFailure


_exiting = False


def set_exiting():
    global _exiting
    _exiting = True


def _match_states():
    print("Starting to match states.")

    try:
        library.match_states()
        server.match_states()

        print("States match.")
    except StateMatchFailure:
        print("Failed to match states.")


def update():
    if not _exiting:
        preferences = addon_preferences.get()
        if preferences:
            server.desired_state.ip = preferences.server_ip
            server.desired_state.location = preferences.server_location
            server.desired_state.port = preferences.server_port

            if server.desired_state.location == 'CUSTOM_CLI':
                server.desired_state.custom_cli_path = preferences.server_custom_cli_path

    _match_states()


class _CurrentState:
    def __init__(self):
        self.library = library.current_state
        self.server = server.current_state


class _DesiredState:
    def __init__(self):
        self.library = library.desired_state
        self.server = server.desired_state


def get_current_state():
    return _CurrentState()


@contextmanager
def update_state():
    try:
        yield _DesiredState()
    finally:
        update()
