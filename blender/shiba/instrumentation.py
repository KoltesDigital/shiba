from dataclasses import dataclass
from shiba import addon_preferences, api, cli_server, \
    server_connection, server_utils


_exiting = False


def set_exiting():
    global _exiting
    _exiting = True


@dataclass
class _State:
    api_loaded: bool = False
    server_custom_cli_path: str = None
    server_location: str = None
    server_started: bool = False


_current_state = _State()
_desired_state = _State()


# TODO improve
def _match_states():
    print("Starting to match states.")

    print(_current_state)
    if _current_state.server_started:
        if _current_state.server_location:
            if _current_state.server_location != 'EXTERNAL':
                cli_server.stop()
            server_connection.disconnect()
        _current_state.server_started = False

    if _desired_state.server_started:
        if _desired_state.server_location != 'EXTERNAL':
            cli_server.start()
        server_connection.connect()
        server_utils.bootstrap()

        _current_state.server_custom_cli_path = \
            _desired_state.server_custom_cli_path
        _current_state.server_location = _desired_state.server_location
        _current_state.server_started = _desired_state.server_started

    if not _current_state.api_loaded and _desired_state.api_loaded:
        api.load()
        _current_state.api_loaded = _desired_state.api_loaded

    if _current_state.api_loaded and not _desired_state.api_loaded:
        api.unload()
        _current_state.api_loaded = _desired_state.api_loaded

    print("States match.")


def update():
    if not _exiting:
        preferences = addon_preferences.get()
        _desired_state.server_location = preferences.server_location \
            if preferences is not None else None
        _desired_state.server_custom_cli_path = \
            preferences.server_custom_cli_path \
            if _desired_state.server_location == 'CUSTOM_CLI' \
            else None

    if _desired_state != _current_state:
        _match_states()


class _ContextManager:
    def __init__(self, state):
        self.__state = state

    def __enter__(self):
        return self.__state

    def __exit__(self, _exc_type, _exc_value, _traceback):
        update()


def state():
    return _ContextManager(_desired_state)
