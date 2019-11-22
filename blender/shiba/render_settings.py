import bpy
from bpy.props import BoolProperty, PointerProperty, StringProperty
from bpy.types import PropertyGroup
from shiba import cli


def _build_executable_update(_self, _context):
    cli.update_build_executable()


def _project_directory_update(_self, _context):
    cli.update_project_directory()


class RenderSettings(PropertyGroup):
    build_executable: BoolProperty(
        name="Build Executable",
        description="In addition to building Blender API",
        default=True,
        update=_build_executable_update
    )

    project_directory: StringProperty(
        name="Project Directory",
        description="Path to a directory which contains shiba.yml",
        subtype='DIR_PATH',
        update=_project_directory_update,
    )

    @classmethod
    def register(cls):
        bpy.types.Scene.shiba = PointerProperty(
            name="Shiba Render Settings",
            description="Shiba render settings",
            type=cls,
        )

    @classmethod
    def unregister(cls):
        del bpy.types.Scene.shiba
