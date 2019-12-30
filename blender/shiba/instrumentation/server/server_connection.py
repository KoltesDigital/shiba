import json


class ServerConnection:
    def __init__(self, socket):
        self.__socket = socket

    def _send_command(self, command):
        message = str.encode(json.dumps(command))
        self.__socket.send(message)
        self.__socket.send(b'\n')

    def send_build_command(self, target):
        self._send_command({
            'command': 'build',
            'target': target,
        })

    def send_set_build_on_change_command(self, executable, library):
        self._send_command({
            'command': 'set-build-on-change',
            'executable': bool(executable),
            'library': bool(library),
        })

    def send_set_project_directory_command(self, path):
        self._send_command({
            'command': 'set-project-directory',
            'path': path,
        })
