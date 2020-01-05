import os
import tempfile
import shutil
from shiba import addon

_temp_dir = os.path.join(tempfile.gettempdir(), addon.name())
if not os.path.exists(_temp_dir):
    os.makedirs(_temp_dir)


class LockedFile:
    def __init__(self, on_opened, on_closed):
        self.__should_open = False
        self.__copied = False
        self.__opened = False

        self.__path = None
        self.__opened_path = None

        self.__on_opened = on_opened
        self.__on_closed = on_closed

    def __del__(self):
        self.close()

    @property
    def is_opened(self):
        return self.__opened

    @property
    def path(self):
        return self.__path

    def _open(self):
        if self.__opened or not self.__path:
            return False

        self.__opened_path = os.path.join(
            _temp_dir, "locked." + os.path.basename(self.__path))

        if not self.__copied:
            try:
                shutil.copy(self.__path, self.__opened_path)
            except FileNotFoundError:
                print('File does not exist: %s.' % self.__path)
                return False
            self.__copied = True

        try:
            self.__on_opened(self.__opened_path)
        except Exception as e:
            print("Failed to open locked file: %s" % e)
            return False

        self.__opened = True
        return True

    def _close(self):
        if not self.__copied:
            return False

        if self.__opened:
            try:
                self.__on_closed()
            except Exception as e:
                print("Failed to close locked file: %s" % e)
                return False

            self.__opened = False

        try:
            os.remove(self.__opened_path)
        except PermissionError:
            pass
        self.__copied = False

        return True

    def set_path(self, path):
        self.__path = path
        self.reload()

    def reload(self):
        must_reload = self.__should_open

        if must_reload:
            self._close()
            return self._open()
        else:
            return True

    def open(self):
        self.__should_open = True
        return self._open()

    def close(self):
        self.__should_open = False
        return self._close()
