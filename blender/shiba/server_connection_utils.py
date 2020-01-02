import bpy
from bpy.app.handlers import persistent
from shiba import instrumentation, path


def _get_project_directory():
    return path.realpath(bpy.context.scene.shiba.project_directory)


def _send_build_commands(server_connection):
    server_connection.send_build_command('library')
    if bpy.context.scene.shiba.build_executable_on_change:
        server_connection.send_build_command('executable')


def bootstrap(server_connection):
    server_connection.send_set_build_on_change_command(
        bpy.context.scene.shiba.build_executable_on_change,
        True,
    )
    server_connection.send_set_project_directory_command(_get_project_directory())
    _send_build_commands(server_connection)


def update_build_on_change(server_connection):
    server_connection.send_set_build_on_change_command(
        bpy.context.scene.shiba.build_executable_on_change,
        True,
    )
    _send_build_commands(server_connection)


def update_project_directory(server_connection):
    server_connection.send_set_project_directory_command(_get_project_directory())
    _send_build_commands(server_connection)


@persistent
def load_handler(_dummy):
    with instrumentation.server.get_server_connection() as server_connection:
        if server_connection:
            update_project_directory(server_connection)


bpy.app.handlers.load_post.append(load_handler)
