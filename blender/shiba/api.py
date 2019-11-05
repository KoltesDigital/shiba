import _ctypes
import ctypes
import os
from shiba.locked_file import LockedFile
import struct
import threading


class API:
    def __init__(self, on_loaded):
        self.__library = None

        self.__on_loaded = on_loaded

        self.__locked_file = LockedFile(self.__run_process, self.__end_process)
        self.__lock = threading.Lock()

    def __run_process(self, path):
        self.__library = ctypes.CDLL(path)
        self.__on_loaded()
        print("API loaded.")

    def __end_process(self):
        _ctypes.FreeLibrary(self.__library._handle)
        del self.__library
        print("API unloaded.")

    def set_path(self, path):
        parent_path = os.path.dirname(path)

        # Make sure indirect DLL dependencies are found.
        if os.environ["PATH"].find(parent_path) == -1:
            os.environ["PATH"] += ";" + parent_path

        with self.__lock:
            self.__locked_file.set_path(path)

    def load(self):
        with self.__lock:
            self.__locked_file.open()

    def unload(self):
        with self.__lock:
            self.__locked_file.close()

    def update(self, time, width, height, is_preview):
        with self.__lock:
            if not self.__locked_file.opened:
                return

            self.__library._shibaUpdate(
                ctypes.c_float(time),
                ctypes.c_int32(width),
                ctypes.c_int32(height),
                ctypes.c_bool(is_preview),
            )

    def render(self, time, width, height, is_preview):
        with self.__lock:
            if not self.__locked_file.opened:
                return

            pixel_count = width * height
            buffer = bytearray(pixel_count * 16)
            Buffer = ctypes.c_char * len(buffer)

            self.__library._shibaRender(
                ctypes.c_float(time),
                ctypes.c_int32(width),
                ctypes.c_int32(height),
                ctypes.c_bool(is_preview),
                Buffer.from_buffer(buffer),
            )

            frame = [None] * pixel_count
            it = struct.iter_unpack('ffff', buffer)
            for i in range(pixel_count):
                frame[i] = next(it)
            return frame

    def viewport_update(self, time, width, height):
        with self.__lock:
            if not self.__locked_file.opened:
                return

            self.__library._shibaViewportUpdate(
                ctypes.c_float(time),
                ctypes.c_int32(width),
                ctypes.c_int32(height),
            )

    def viewport_render(self, time, width, height):
        with self.__lock:
            if not self.__locked_file.opened:
                return

            self.__library._shibaViewportRender(
                ctypes.c_float(time),
                ctypes.c_int32(width),
                ctypes.c_int32(height),
            )
