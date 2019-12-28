import bpy
from bpy.app.handlers import persistent
import os.path
from shiba import server_connection


def _get_project_directory():
    path = bpy.context.scene.shiba.project_directory
    path = os.path.realpath(bpy.path.abspath(path))
    return path


def _send_build_commands(library_mode):
    server_connection.send_build_command(library_mode, 'library')
    if bpy.context.scene.shiba.build_executable_on_change:
        server_connection.send_build_command('full', 'executable')


def bootstrap():
    server_connection.send_set_build_mode_on_change_command(
        'full' if bpy.context.scene.shiba.build_executable_on_change else None,
        'updates',
    )
    server_connection.send_set_project_directory_command(
        _get_project_directory())
    _send_build_commands('full')


def update_build_on_change():
    server_connection.send_set_build_mode_on_change_command(
        'full' if bpy.context.scene.shiba.build_executable_on_change else None,
        'updates',
    )
    _send_build_commands('updates')


def update_project_directory():
    server_connection.send_set_project_directory_command(
        _get_project_directory())
    _send_build_commands('updates')


@persistent
def load_handler(_dummy):
    update_project_directory()


bpy.app.handlers.load_post.append(load_handler)
