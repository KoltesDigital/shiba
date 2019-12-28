import json
import socket
from shiba import addon_preferences, api, notifications, uniforms
from shiba.notifications import Notification
from threading import Lock, Thread

_building_count = 0
_building_lock = Lock()

_socket = None
_socket_lock = Lock()
_socket_thread = None


def _handle_event_build_ended(obj):
    global _building_count
    global _building_notification

    with _building_lock:
        _building_count -= 1
        if _building_count == 0:
            notifications.remove(_building_notification)

    print("Build duration: %fs." % obj['duration'])


def _handle_event_build_started(obj):
    global _building_count
    global _building_notification

    with _building_lock:
        if _building_count == 0:
            _building_notification = Notification("Building...")
            notifications.add(_building_notification)
        _building_count += 1


def _handle_event_executable_compiled(obj):
    size = obj['size']
    notifications.add(Notification("Executable size: %d." % size, 5))


def _handle_event_error(obj):
    print("Server error: %s" % obj['message'])


def _handle_event_library_compiled(obj):
    api.set_path(obj['path'])
    count, descriptors = api.get_active_uniform_descriptors()
    uniforms.set_api_active_uniform_descriptors(count, descriptors)


def _handle_event_shader_passes_generated(obj):
    api.set_shader_passes(obj['passes'])


_event_handlers = {
    'build-ended': _handle_event_build_ended,
    'build-started': _handle_event_build_started,
    'executable-compiled': _handle_event_executable_compiled,
    'error': _handle_event_error,
    'library-compiled': _handle_event_library_compiled,
    'shader-passes-generated': _handle_event_shader_passes_generated,
}


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
            event_handler = _event_handlers.get(event_kind, None)
            if event_handler:
                event_handler(obj)
            else:
                print("No handler for event %s." % event_kind)

            buffer = buffer[index + 1:]
            index = buffer.find(b'\n')


def connect():
    global _socket
    global _socket_thread
    with _socket_lock:
        if not _socket:
            print("Connecting to server.")
            preferences = addon_preferences.get()
            _socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            try:
                _socket.connect(
                    (preferences.server_ip, preferences.server_port))
            except ConnectionRefusedError:
                _socket = None
                print("Failed to connect to %s:%d" %
                      (preferences.server_ip, preferences.server_port))
                return
            _socket_thread = Thread(
                target=_run_socket_thread
            )
            _socket_thread.start()
            print("Connected to server.")


def disconnect():
    global _socket
    global _socket_thread
    with _socket_lock:
        if _socket:
            print("Disconnecting from server.")
            _socket.close()
            _socket_thread.join()
            _socket_thread = None
            _socket = None
            print("Disconnected from server.")


def _send_command(command):
    with _socket_lock:
        if _socket:
            message = str.encode(json.dumps(command))
            _socket.send(message)
            _socket.send(b'\n')


def send_build_command(mode, target):
    _send_command({
        'command': 'build',
        'mode': mode,
        'target': target,
    })


def send_set_build_mode_on_change_command(executable, library):
    command = {
        'command': 'set-build-mode-on-change',
    }
    if executable:
        command['executable'] = executable
    if library:
        command['library'] = library
    _send_command(command)


def send_set_project_directory_command(path):
    _send_command({
        'command': 'set-project-directory',
        'path': path,
    })
