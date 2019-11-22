import bpy
from bpy.app.handlers import persistent
import os.path
from shiba import server_connection


def _get_project_directory():
    path = bpy.context.scene.shiba.project_directory
    path = os.path.realpath(bpy.path.abspath(path))
    return path


def bootstrap():
    server_connection.send_get_blender_api_path_command()
    server_connection.send_set_build_executable_command(
        bpy.context.scene.shiba.build_executable)
    server_connection.send_set_project_directory_command(
        _get_project_directory())
    server_connection.send_build_command()


def update_build_executable():
    server_connection.send_set_build_executable_command(
        bpy.context.scene.shiba.build_executable)
    server_connection.send_build_command()


def update_project_directory():
    server_connection.send_set_project_directory_command(
        _get_project_directory())
    server_connection.send_build_command()


@persistent
def load_handler(_dummy):
    update_project_directory()


bpy.app.handlers.load_post.append(load_handler)
