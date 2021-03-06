import ctypes
from dataclasses import dataclass
from shiba import callback_lists
from shiba.library_definitions import ShaderProgram
import struct
from typing import Optional


@dataclass
class _ToUpdate:
    shader_programs: Optional[dict] = None


class LibraryWrapper:
    def __init__(self, library):
        self.__library = library

        self.__to_update = _ToUpdate()
        self.__viewport_to_update = _ToUpdate()

    def set_shader_programs(self, programs):
        self.__to_update.shader_programs = programs
        self.__viewport_to_update.shader_programs = programs
        callback_lists.viewport_update.trigger()

    def _execute_updates(self, to_update):
        if to_update.shader_programs:
            def to_char_p(d, key):
                value = d.get(key, None)
                if value is not None:
                    value = ctypes.c_char_p(value.encode('utf-8'))
                else:
                    value = ctypes.c_char_p()
                return value

            count = len(to_update.shader_programs)
            ShaderProgramArray = ShaderProgram * count
            array = [ShaderProgram(
                vertex=to_char_p(shader_program, 'vertex'),
                fragment=to_char_p(shader_program, 'fragment'),
            )
                for shader_program in to_update.shader_programs]
            shader_programs = ShaderProgramArray(*array)
            self.__library._shibaUpdatePrograms(
                count,
                shader_programs,
            )
            to_update.shader_programs = None

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
