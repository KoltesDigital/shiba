import json
from shiba import paths
from shiba.api import API
from shiba.locked_file import LockedFile
import socket
import subprocess
import threading


class Tool:
    def __init__(self, on_api_loaded):
        self.__process = None
        self.__process_thread = None

        self.__socket = None
        self.__socket_thread = None

        self.__api = API(on_api_loaded)

        self.__locked_file = LockedFile(self.__run_process, self.__end_process)
        self.__lock = threading.Lock()

    @property
    def api(self):
        return self.__api

    def with_api(self, cb):
        with self.__lock:
            cb(self.__api)

    def __run_process_thread(self):
        while True:
            line = self.__process.stdout.readline()
            if self.__process.poll() is not None:
                break
            print('Shiba: %s' % line.decode().rstrip())
        rc = self.__process.poll()

        print('Shiba exited with code %d' % rc)

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
                    self.__api.set_path(obj['path'])
                if event == 'error':
                    print('Error: %s' % obj['message'])

                buffer = buffer[index + 1:]
                index = buffer.find(b'\n')

    def __run_process(self, path):
        self.__api.load()

        self.__process = subprocess.Popen(
            [path, 'server'],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        self.__process_thread = threading.Thread(
            target=self.__run_process_thread
        )
        self.__process_thread.start()

        self.__socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.__socket.connect(('127.0.0.1', 5184))
        self.__socket_thread = threading.Thread(
            target=self.__run_socket_thread
        )
        self.__socket_thread.start()

        print("Shiba started.")

    def __end_process(self):
        self.__process.terminate()
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
                self.__socket.send(b'{"command":"build"}\n')
