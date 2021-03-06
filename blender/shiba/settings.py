import bpy
from shiba import instrumentation, server_connection_utils
from shiba.uniforms import Uniforms


def _build_executable_on_change_update(_self, _context):
    with instrumentation.server.get_server_connection() as server_connection:
        if server_connection:
            server_connection_utils.update_build_on_change(server_connection)


def _project_directory_update(_self, _context):
    with instrumentation.server.get_server_connection() as server_connection:
        if server_connection:
            server_connection_utils.update_project_directory(server_connection)


class Settings(bpy.types.PropertyGroup):
    build_executable_on_change: bpy.props.BoolProperty(
        description="Automatically build executable when a file is modified",
        default=True,
        name="Build Executable On Change",
        update=_build_executable_on_change_update
    )

    export_directory: bpy.props.StringProperty(
        default='//export',
        description="Path to a directory which will contain the exported build",
        name="Export Directory",
        subtype='DIR_PATH',
    )

    export_output: bpy.props.EnumProperty(
        default='directory',
        items=[
            (
                'directory',
                "Directory",
                "Outputs a directory.",
            ),
            (
                '7z',
                "7z",
                "Outputs a 7z archive.",
            ),
            (
                'zip',
                "Zip",
                "Outputs a Zip archive.",
            ),
        ],
        name="Export Output",
    )

    project_directory: bpy.props.StringProperty(
        description="Path to a directory which contains shiba.yml",
        default='//project',
        name="Project Directory",
        subtype='DIR_PATH',
        update=_project_directory_update,
    )

    uniforms: bpy.props.PointerProperty(type=Uniforms)

    @classmethod
    def register(cls):
        bpy.types.Scene.shiba = bpy.props.PointerProperty(
            description="Shiba settings",
            name="Shiba Settings",
            type=cls,
        )

    @classmethod
    def unregister(cls):
        del bpy.types.Scene.shiba
