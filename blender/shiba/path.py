import bpy
import os.path


def realpath(path):
    return os.path.realpath(bpy.path.abspath(path))
