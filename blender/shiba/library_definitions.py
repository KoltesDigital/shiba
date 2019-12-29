import ctypes


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
