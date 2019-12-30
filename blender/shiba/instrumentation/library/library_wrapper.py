import ctypes
from shiba import callback_lists
from shiba.library_definitions import ShaderPasses
import struct


class _ToUpdate:
    def __init__(self):
        self.shader_passes = None


class LibraryWrapper:
    def __init__(self, library):
        self.__library = library

        self.__to_update = _ToUpdate()
        self.__viewport_to_update = _ToUpdate()

    def set_shader_passes(self, passes):
        self.__to_update.shader_passes = passes
        self.__viewport_to_update.shader_passes = passes
        callback_lists.viewport_update.trigger()

    def _execute_updates(self, to_update):
        if to_update.shader_passes:
            def to_char_p(d, key):
                value = d.get(key, None)
                if value is not None:
                    value = ctypes.c_char_p(value.encode('utf-8'))
                else:
                    value = ctypes.c_char_p()
                return value

            count = len(to_update.shader_passes)
            ShaderPassArray = ShaderPasses * count
            array = [ShaderPasses(
                vertex=to_char_p(shader_pass, 'vertex'),
                fragment=to_char_p(shader_pass, 'fragment'),
            )
                for shader_pass in to_update.shader_passes]
            passes = ShaderPassArray(*array)
            self.__library._shibaUpdateShaderPasses(
                count,
                passes,
            )
            to_update.shader_passes = None

    def update(self, width, height, is_preview):
        self.__library._shibaUpdate(
            ctypes.c_int32(width),
            ctypes.c_int32(height),
            ctypes.c_bool(is_preview),
        )

    def render(self, width, height, is_preview):
        self.__library._shibaEnsureIsInitialized(
            ctypes.c_int32(width),
            ctypes.c_int32(height),
        )

        self._execute_updates(self.__to_update)

        pixel_count = width * height
        buffer = bytearray(pixel_count * 16)
        Buffer = ctypes.c_char * len(buffer)

        self.__library._shibaRender(
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

    def viewport_update(self, width, height):
        self.__library._shibaViewportUpdate(
            ctypes.c_int32(width),
            ctypes.c_int32(height),
        )

    def viewport_render(self, width, height):
        self.__library._shibaViewportEnsureIsInitialized(
            ctypes.c_int32(width),
            ctypes.c_int32(height),
        )

        self._execute_updates(self.__viewport_to_update)

        self.__library._shibaViewportRender(
            ctypes.c_int32(width),
            ctypes.c_int32(height),
        )

    def set_uniform_values(self, uniform_values):
        self.__library._shibaSetActiveUniformValues(uniform_values)
