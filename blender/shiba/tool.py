import bpy
from bpy.app.handlers import persistent
from itertools import filterfalse
import json
import os.path
from shiba import paths
from shiba.api import API
from shiba.locked_file import LockedFile
import socket
import subprocess
from threading import Lock, Thread, main_thread
import time


class _Tool:
    def __init__(self, on_api_changed):
        self.__process = None
        self.__process_thread = None

        self.__socket = None
        self.__socket_thread = None

        self.__api = API(on_api_changed)

        self.__locked_file = LockedFile(self.__run_process, self.__end_process)
        self.__lock = Lock()

    @property
    def api(self):
        return self.__api

    def with_api(self, cb):
        with self.__lock:
            cb(self.__api)

    def __run_process_thread(self):
        while True:
            try:
                line = self.__process.stdout.readline()
                print('Shiba: %s' % line.decode().rstrip())
            except ValueError:
                if self.__process.poll() is not None:
                    break

        rc = self.__process.poll()
        print('Shiba exited with code %d.' % rc)

    def __run_socket_thread(self):
        buffer = bytearray()
        while True:
            try:
                chunk = self.__socket.recv(1024)
            except ConnectionResetError:
                break

            if not chunk:
                break
            buffer.extend(chunk)
            index = buffer.find(b'\n')
            while index >= 0:
                line = buffer[:index]

                obj = json.loads(line)
                event = obj['event']
                if event == "blender-api-available":
                    self.__api.reload()
                if event == "blender-api-path":
                    self.__api.set_path(obj['path'])
                if event == 'error':
                    print('Error: %s' % obj['message'])
                if event == 'shader-passes-available':
                    self.__api.set_shader_passes(obj['passes'])

                buffer = buffer[index + 1:]
                index = buffer.find(b'\n')

    def __run_process(self, path):
        self.__api.load()

        self.__process = subprocess.Popen(
            [path, 'server'],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        self.__process_thread = Thread(
            target=self.__run_process_thread
        )
        self.__process_thread.start()

        self.__socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.__socket.connect(('127.0.0.1', 5184))
        self.__socket_thread = Thread(
            target=self.__run_socket_thread
        )
        self.__socket_thread.start()

        self.__socket.send(b'{"command":"get-blender-api-path"}\n')

        print("Shiba started.")

    def __end_process(self):
        print("Exiting Shiba.")
        self.__process.terminate()

        try:
            self.__process.communicate(timeout=15)
        except subprocess.TimeoutExpired:
            print("Forcing Shiba to exit.")
            self.__process.kill()
            self.__process.communicate()

        self.__process_thread.join()
        self.__process_thread = None
        self.__process = None

        self.__socket.close()
        self.__socket_thread.join()
        self.__socket_thread = None
        self.__socket = None

        self.__api.unload()

        print("Shiba stopped.")

    def update_path(self):
        with self.__lock:
            self.__locked_file.set_path(paths.shiba())

    def start(self):
        with self.__lock:
            self.__locked_file.open()

    def stop(self):
        with self.__lock:
            self.__locked_file.close()

    def build(self):
        with self.__lock:
            if self.__socket:
                self.__socket.send(b'{"command":"build-blender-api"}\n')

    def set_project_directory(self, path):
        with self.__lock:
            if self.__socket:
                message = {
                    "command": "set-project-directory",
                    "path": path,
                }
                message_as_bytes = str.encode(json.dumps(message))
                self.__socket.send(message_as_bytes)
                self.__socket.send(b'\n')


def _call_callback_and_should_be_removed(callback):
    try:
        callback()
        return False
    except ReferenceError:
        return True


def _call_api_changed_callbacks():
    _on_api_changed_callbacks[:] = filterfalse(
        _call_callback_and_should_be_removed, _on_api_changed_callbacks)


def register_api_changed_callback(on_api_changed):
    _on_api_changed_callbacks.append(on_api_changed)


def is_active():
    return _instance is not None


def _get_project_path():
    path = bpy.context.scene.shiba.project_path
    if os.path.isabs(path):
        path = os.path.join(bpy.data.filepath, path)
    return path


def instance():
    global _instance
    if _instance is None:
        _instance = _Tool(_call_api_changed_callbacks)
        _instance.update_path()
        _instance.start()
        _instance.set_project_directory(_get_project_path())
        _instance.build()
    return _instance


def update_path():
    if is_active():
        i = instance()
        i.set_project_directory(_get_project_path())
        i.build()


def _stop():
    global _instance
    if _instance is not None:
        instance = _instance
        _instance = None
        _on_api_changed_callbacks.clear()
        instance.stop()


def _run_thread():
    while True:
        if not main_thread().is_alive():
            break
        time.sleep(1)

    _stop()


def unregister():
    _stop()


@persistent
def load_handler(_dummy):
    update_path()


_on_api_changed_callbacks = []
_instance = None

bpy.app.handlers.load_post.append(load_handler)

_thread = Thread(
    target=_run_thread
)
_thread.start()
