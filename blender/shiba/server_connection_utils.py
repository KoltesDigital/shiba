import bpy
from bpy.app.handlers import persistent
import os.path
from shiba import instrumentation


def _get_project_directory():
    path = bpy.context.scene.shiba.project_directory
    path = os.path.realpath(bpy.path.abspath(path))
    return path


def _send_build_commands(server_connection, library_mode):
    server_connection.send_build_command(library_mode, 'library')
    if bpy.context.scene.shiba.build_executable_on_change:
        server_connection.send_build_command('full', 'executable')


def bootstrap(server_connection):
    server_connection.send_set_build_on_change_command(
        bpy.context.scene.shiba.build_executable_on_change,
        True,
    )
    server_connection.send_set_project_directory_command(
        _get_project_directory())
    _send_build_commands(server_connection, 'full')


def update_build_on_change(server_connection):
    server_connection.send_set_build_on_change_command(
        bpy.context.scene.shiba.build_executable_on_change,
        True,
    )
    _send_build_commands(server_connection, 'updates')


def update_project_directory(server_connection):
    server_connection.send_set_project_directory_command(
        _get_project_directory())
    _send_build_commands(server_connection, 'updates')


@persistent
def load_handler(_dummy):
    with instrumentation.server.get_server_connection() as server_connection:
        if server_connection:
            update_project_directory(server_connection)


bpy.app.handlers.load_post.append(load_handler)