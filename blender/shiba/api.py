import _ctypes
import bpy
import ctypes
import os
import shutil
import struct
import tempfile
import threading


class API:
    def __init__(self):
        self.__handle = None
        self.__dll = None

        # Development path.
        addons_to_load = os.environ.get('ADDONS_TO_LOAD')
        if addons_to_load is not None:
            script_dir = os.path.join(tempfile.gettempdir(), "shiba")
        else:
            script_dir = os.path.dirname(__file__)

        # make sure indirect DLL dependencies are found
        if os.environ["PATH"].find(script_dir) == -1:
            os.environ["PATH"] += ";" + script_dir

        self.__dll_name = os.path.join(script_dir, "blender_api.dll")
        self.__loaded_dll_name = os.path.join(
            bpy.app.tempdir, "shiba.loaded.dll")

        self.__lock = threading.Lock()

    def _load_library(self):
        if not self.__dll:
            # copy the dll, to allow hot reload after rebuild
            shutil.copy(self.__dll_name, self.__loaded_dll_name)

            # loading pattern from
            # http://stackoverflow.com/questions/21770419/free-the-opened-ctypes-library-in-python
            self.__dll = ctypes.CDLL(self.__loaded_dll_name)

    def _unload_library(self):
        if self.__dll:
            _ctypes.FreeLibrary(self.__dll._handle)
            del self.__dll

            # remove the temp dll
            os.remove(self.__loaded_dll_name)

    def load(self):
        try:
            self.__lock.acquire()
            self._load_library()
        finally:
            self.__lock.release()

    def unload(self):
        try:
            self.__lock.acquire()
            self._unload_library()
        finally:
            self.__lock.release()

    def reload(self):
        try:
            self.__lock.acquire()
            self._unload_library()
            self._load_library()
        finally:
            self.__lock.release()

    def update(self, time, width, height, is_preview):
        try:
            self.__lock.acquire()
            self.__dll._shibaUpdate(
                ctypes.c_float(time),
                ctypes.c_int32(width),
                ctypes.c_int32(height),
                ctypes.c_bool(is_preview),
            )
        finally:
            self.__lock.release()

    def render(self, time, width, height, is_preview):
        pixel_count = width * height
        buffer = bytearray(pixel_count * 16)
        Buffer = ctypes.c_char * len(buffer)
        try:
            self.__lock.acquire()
            self.__dll._shibaRender(
                ctypes.c_float(time),
                ctypes.c_int32(width),
                ctypes.c_int32(height),
                ctypes.c_bool(is_preview),
                Buffer.from_buffer(buffer),
            )
        finally:
            self.__lock.release()
        frame = [None] * pixel_count
        it = struct.iter_unpack('ffff', buffer)
        for i in range(pixel_count):
            frame[i] = next(it)
        return frame

    def viewport_update(self, time, width, height):
        try:
            self.__lock.acquire()
            self.__dll._shibaViewportUpdate(
                ctypes.c_float(time),
                ctypes.c_int32(width),
                ctypes.c_int32(height),
            )
        finally:
            self.__lock.release()

    def viewport_render(self, time, width, height):
        try:
            self.__lock.acquire()
            self.__dll._shibaViewportRender(
                ctypes.c_float(time),
                ctypes.c_int32(width),
                ctypes.c_int32(height),
            )
        finally:
            self.__lock.release()
