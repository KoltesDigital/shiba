import json


class ServerConnection:
    def __init__(self, socket):
        self.__socket = socket

    def _send_command(self, command):
        message = str.encode(json.dumps(command))
        self.__socket.send(message)
        self.__socket.send(b'\n')

    def send_build_command(self, mode, target):
        self._send_command({
            'command': 'build',
            'mode': mode,
            'target': target,
        })

    def send_set_build_mode_on_change_command(self, executable, library):
        command = {
            'command': 'set-build-mode-on-change',
        }
        if executable:
            command['executable'] = executable
        if library:
            command['library'] = library
        self._send_command(command)

    def send_set_project_directory_command(self, path):
        self._send_command({
            'command': 'set-project-directory',
            'path': path,
        })
