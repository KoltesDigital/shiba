import json
import socket
from shiba import api, notifications
from shiba.notifications import Notification
from threading import Lock, Thread

_building_count = None
_building_lock = None

_socket = None
_socket_thread = None


def _handle_event(obj):
    global _building_count
    global _building_notification

    event = obj['event']

    if event == "blender-api-path":
        api.set_path(obj['path'])

    if event == "build-started":
        with _building_lock:
            if _building_count == 0:
                what = obj['what']
                what_str = " + ".join(
                    map(lambda what_item: what_item['kind'], what))
                _building_notification = Notification(
                    "Building %s..." % what_str)
                notifications.add(_building_notification)
            _building_count += 1

    if event == "build-ended":
        with _building_lock:
            _building_count -= 1
            if _building_count == 0:
                notifications.remove(_building_notification)
        what = obj['what']
        for what_item in what:
            kind = what_item['kind']
            if kind == "blender-api":
                api.reload()
            if kind == "executable":
                with _lock:
                    _socket.send(
                        b'{"command":"get-executable-size"}\n')
            if kind == 'shader-passes':
                api.set_shader_passes(what_item['passes'])

    if event == 'error':
        print('Server error: %s' % obj['message'])

    if event == "executable-size":
        size = obj['size']
        add_notification(Notification("Executable size: %d" % size, 5))


def _run_socket_thread():
    buffer = bytearray()
    while True:
        try:
            chunk = _socket.recv(1024)
        except ConnectionResetError:
            break

        if not chunk:
            break
        buffer.extend(chunk)
        index = buffer.find(b'\n')
        while index >= 0:
            line = buffer[:index]

            obj = json.loads(line)
            _handle_event(obj)

            buffer = buffer[index + 1:]
            index = buffer.find(b'\n')


_socket = None
_socket_thread = None


def connect():
    global _socket
    global _socket_thread
    if not _socket:
        print("Connecting to server.")
        _socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        _socket.connect(('127.0.0.1', 5184))
        _socket_thread = Thread(
            target=_run_socket_thread
        )
        _socket_thread.start()
        print("Connected to server.")


def disconnect():
    global _socket
    global _socket_thread
    if _socket:
        print("Disconnecting from server.")
        _socket.close()
        _socket_thread.join()
        _socket_thread = None
        _socket = None
        print("Disconnected from server.")


def send_build_command():
    if _socket:
        _socket.send(b'{"command":"build"}\n')


def send_get_blender_api_path_command():
    if _socket:
        _socket.send(b'{"command":"get-blender-api-path"}\n')


def send_set_build_executable_command(build_executable):
    if _socket:
        message = {
            "command": "set-build-executable",
            "build-executable": build_executable,
        }
        message_as_bytes = str.encode(json.dumps(message))
        _socket.send(message_as_bytes)
        _socket.send(b'\n')


def send_set_project_directory_command(path):
    if _socket:
        message = {
            "command": "set-project-directory",
            "path": path,
        }
        message_as_bytes = str.encode(json.dumps(message))
        _socket.send(message_as_bytes)
        _socket.send(b'\n')


def register():
    global _building_count
    global _building_lock

    _building_count = 0
    _building_lock = Lock()
