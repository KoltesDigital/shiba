import ctypes


Matrix = ctypes.c_float * 16


def to_c_matrix(matrix):
    return Matrix(
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


class UniformValue(ctypes.Structure):
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
