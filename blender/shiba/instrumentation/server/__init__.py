from contextlib import contextmanager
from dataclasses import dataclass
import json
from shiba import addon_preferences, server_connection_utils
from shiba.instrumentation import paths
from shiba.instrumentation.error import StateMatchFailure
from shiba.instrumentation.server import cli_server
from shiba.instrumentation.server.desired_state import desired_state
from shiba.instrumentation.server.event_handlers import EVENT_HANDLERS
from shiba.instrumentation.server.server_connection import ServerConnection
import socket
from threading import Lock, Thread


@dataclass
class _CurrentState:
    custom_cli_path: str = None
    ip: str = None
    location: str = None
    port: int = None

    @property
    def connected(self):
        return _socket


current_state = _CurrentState()


_lock = Lock()
_server_connection = None
_socket = None
_socket_thread = None


def _run_socket_thread():
    buffer = bytearray()
    while True:
        try:
            chunk = _socket.recv(1024)
        except ConnectionAbortedError:
            break
        except ConnectionResetError:
            break

        if not chunk:
            break
        buffer.extend(chunk)
        index = buffer.find(b'\n')
        while index >= 0:
            line = buffer[:index]

            obj = json.loads(line)
            event_kind = obj['event']
            event_handler = EVENT_HANDLERS.get(event_kind, None)
            if event_handler:
                event_handler(obj)
            else:
                print("No handler for event %s." % event_kind)

            buffer = buffer[index + 1:]
            index = buffer.find(b'\n')


def _connect():
    global _server_connection
    global _socket
    global _socket_thread

    print("Connecting to server.")

    preferences = addon_preferences.get()
    _socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        _socket.connect(
            (preferences.server_ip, preferences.server_port))
    except ConnectionRefusedError:
        _socket = None
        print("Failed to connect to %s:%d." %
              (preferences.server_ip, preferences.server_port))
        raise StateMatchFailure()

    _socket_thread = Thread(
        target=_run_socket_thread
    )
    _socket_thread.start()

    _server_connection = ServerConnection(_socket)

    print("Connected to server.")


def _disconnect():
    global _server_connection
    global _socket
    global _socket_thread

    print("Disconnecting from server.")

    _socket.close()
    _socket = None

    _socket_thread.join()
    _socket_thread = None

    _server_connection = None

    print("Disconnected from server.")


def match_states():
    with _lock:
        server_options_mismatch = current_state.ip != desired_state.ip \
            or current_state.location != desired_state.location \
            or current_state.port != desired_state.port
        if not server_options_mismatch and current_state.location == 'CUSTOM_CLI':
            server_options_mismatch = current_state.custom_cli_path != desired_state.custom_cli_path

        if _socket and (not desired_state.connected or server_options_mismatch):
            if current_state.location != 'EXTERNAL':
                cli_server.locked_file.close()
            _disconnect()

        if desired_state.connected and not _socket:
            if desired_state.location != 'EXTERNAL':
                if current_state.location == 'CUSTOM_CLI':
                    cli_server.locked_file.set_path(desired_state.custom_cli_path)
                else:
                    cli_server.locked_file.set_path(paths.cli())
                result = cli_server.locked_file.open()
                if not result:
                    raise StateMatchFailure()
            _connect()
            server_connection_utils.bootstrap(_server_connection)

            current_state.ip = desired_state.ip
            current_state.location = desired_state.location
            current_state.port = desired_state.port

            if current_state.location == 'CUSTOM_CLI':
                current_state.custom_cli_path = desired_state.custom_cli_path


@contextmanager
def get_server_connection():
    with _lock:
        yield _server_connection
