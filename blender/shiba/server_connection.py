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


def _handle_event(obj):
    global _building_count
    global _building_notification

    event = obj['event']

    if event == "blender-api-path":
        api.set_path(obj['path'])
        count, descriptors = api.get_active_uniform_descriptors()
        uniforms.set_api_active_uniform_descriptors(count, descriptors)

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
                with _socket_lock:
                    _socket.send(
                        b'{"command":"get-executable-size"}\n')
            if kind == 'shader-passes':
                api.set_shader_passes(what_item['passes'])

    if event == 'error':
        print('Server error: %s' % obj['message'])

    if event == "executable-size":
        size = obj['size']
        notifications.add(Notification("Executable size: %d" % size, 5))


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
            _handle_event(obj)

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


def send_build_command():
    with _socket_lock:
        if _socket:
            _socket.send(b'{"command":"build"}\n')


def send_get_blender_api_path_command():
    with _socket_lock:
        if _socket:
            _socket.send(b'{"command":"get-blender-api-path"}\n')


def send_set_build_executable_command(build_executable):
    with _socket_lock:
        if _socket:
            message = {
                "command": "set-build-executable",
                "build-executable": build_executable,
            }
            message_as_bytes = str.encode(json.dumps(message))
            _socket.send(message_as_bytes)
            _socket.send(b'\n')


def send_set_project_directory_command(path):
    with _socket_lock:
        if _socket:
            message = {
                "command": "set-project-directory",
                "path": path,
            }
            message_as_bytes = str.encode(json.dumps(message))
            _socket.send(message_as_bytes)
            _socket.send(b'\n')
