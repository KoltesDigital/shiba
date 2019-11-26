import _ctypes
import ctypes
import os
from shiba import callback_lists
from shiba.locked_file import LockedFile
import struct
from threading import Lock

UNIFORM_ANNOTATION_KIND_CONTROL = 0


class UniformAnnotationDescriptor(ctypes.Structure):
    _fields_ = [
        ('kind', ctypes.c_int),
    ]


UNIFORM_ANNOTATION_CONTROL_KIND_CHECKBOX = 0
UNIFORM_ANNOTATION_CONTROL_KIND_OBJECT = 1
UNIFORM_ANNOTATION_CONTROL_KIND_SLIDER = 2


class UniformAnnotationControlDescriptor(ctypes.Structure):
    _fields_ = [
        ('kind', ctypes.c_int),
        ('control_kind', ctypes.c_int),
        ('control_parameters', ctypes.c_char_p),
    ]


class UniformDescriptor(ctypes.Structure):
    _fields_ = [
        ('annotation_count', ctypes.c_int),
        ('annotations', ctypes.POINTER(
            ctypes.POINTER(UniformAnnotationDescriptor))),
        ('name', ctypes.c_char_p),
        ('type_name', ctypes.c_char_p),
    ]


class UniformValue(ctypes.Union):
    _fields_ = [
        ('as_float', ctypes.c_float),
        ('as_int', ctypes.c_int),
        ('as_mat2', ctypes.c_float * 4),
        ('as_mat3', ctypes.c_float * 9),
        ('as_mat4', ctypes.c_float * 16),
        ('as_uint', ctypes.c_uint),
    ]


class ShaderPasses(ctypes.Structure):
    _fields_ = [
        ('vertex', ctypes.c_char_p),
        ('fragment', ctypes.c_char_p),
    ]


_library = None


def _load_library(path):
    global _library
    print("Loading API.")
    _library = ctypes.CDLL(path)
    callback_lists.viewport_update.trigger()
    print("API loaded.")


def _close_library():
    global _library
    print("Unloading API.")
    _ctypes.FreeLibrary(_library._handle)
    _library = None
    print("API unloaded.")


_locked_file = LockedFile(_load_library, _close_library)
_lock = Lock()


def load():
    with _lock:
        _locked_file.open()


def unload():
    with _lock:
        _locked_file.close()


def reload():
    with _lock:
        _locked_file.reload()


def is_loaded():
    with _lock:
        return _locked_file.is_opened


def set_path(path):
    parent_path = os.path.dirname(path)

    # Make sure indirect DLL dependencies are found.
    if os.environ["PATH"].find(parent_path) == -1:
        os.environ["PATH"] += ";" + parent_path

    with _lock:
        _locked_file.set_path(path)
        callback_lists.viewport_update.trigger()


class _ToUpdate:
    def __init__(self):
        self.object_instances = None
        self.shader_passes = None


_to_update = _ToUpdate()
_viewport_to_update = _ToUpdate()


def set_object_instances(object_instances):
    with _lock:
        _to_update.object_instances = object_instances
        _viewport_to_update.object_instances = object_instances
        callback_lists.viewport_update.trigger()


def set_shader_passes(passes):
    with _lock:
        _to_update.shader_passes = passes
        _viewport_to_update.shader_passes = passes
        callback_lists.viewport_update.trigger()


def _execute_updates(to_update):
    if to_update.object_instances:
        for object_instance in to_update.object_instances:
            _library._shibaUpdateObject(
                ctypes.c_char_p(
                    object_instance.object.name.encode('utf-8')),
                _to_c_matrix(object_instance.matrix_world),
            )
        to_update.object_instances = None

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
        _library._shibaUpdateShaderPasses(
            count,
            passes,
        )
        to_update.shader_passes = None


def update(time, width, height, is_preview):
    with _lock:
        if not _locked_file.is_opened:
            return

        _library._shibaUpdate(
            ctypes.c_float(time),
            ctypes.c_int32(width),
            ctypes.c_int32(height),
            ctypes.c_bool(is_preview),
        )


def render(time, width, height, is_preview):
    with _lock:
        if not _locked_file.is_opened:
            return

        _library._shibaEnsureIsInitialized(
            ctypes.c_int32(width),
            ctypes.c_int32(height),
        )

        _execute_updates(_to_update)

        pixel_count = width * height
        buffer = bytearray(pixel_count * 16)
        Buffer = ctypes.c_char * len(buffer)

        _library._shibaRender(
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


def viewport_update(time, width, height):
    with _lock:
        if not _locked_file.is_opened:
            return

        _library._shibaViewportUpdate(
            ctypes.c_float(time),
            ctypes.c_int32(width),
            ctypes.c_int32(height),
        )


def viewport_render(time, width, height):
    with _lock:
        if not _locked_file.is_opened:
            return

        _library._shibaViewportEnsureIsInitialized(
            ctypes.c_int32(width),
            ctypes.c_int32(height),
        )

        _execute_updates(_viewport_to_update)

        _library._shibaViewportRender(
            ctypes.c_float(time),
            ctypes.c_int32(width),
            ctypes.c_int32(height),
        )


def set_override_matrices(
    view_matrix, inv_view_matrix,
    projection_matrix
):
    with _lock:
        if not _locked_file.is_opened:
            return

        _library._shibaSetOverrideMatrices(
            _to_c_matrix(view_matrix),
            _to_c_matrix(inv_view_matrix),
            _to_c_matrix(projection_matrix),
            _to_c_matrix(projection_matrix.inverted()),
        )


def get_active_uniform_descriptors():
    with _lock:
        if not _locked_file.is_opened:
            return 0, None

        active_uniform_count = ctypes.c_int()
        active_uniform_descriptors = ctypes.POINTER(UniformDescriptor)()
        _library._shibaGetActiveUniformDescriptors(
            ctypes.byref(active_uniform_count),
            ctypes.byref(active_uniform_descriptors),
        )
        return active_uniform_count.value, active_uniform_descriptors


def set_uniform_values(uniform_values):
    with _lock:
        if not _locked_file.is_opened:
            return

        _library._shibaSetActiveUniformValues(uniform_values)


_Matrix = ctypes.c_float * 16


def _to_c_matrix(matrix):
    return _Matrix(
        matrix[0][0],
        matrix[1][0],
        matrix[2][0],
        matrix[3][0],
        matrix[0][1],
        matrix[1][1],
        matrix[2][1],
        matrix[3][1],
        matrix[0][2],
        matrix[1][2],
        matrix[2][2],
        matrix[3][2],
        matrix[0][3],
        matrix[1][3],
        matrix[2][3],
        matrix[3][3],
    )
