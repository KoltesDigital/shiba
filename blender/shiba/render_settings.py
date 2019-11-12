import bpy
from bpy.props import PointerProperty, StringProperty
from bpy.types import PropertyGroup
from shiba import tool


def _update(_self, _context):
    tool.update_path()


class RenderSettings(PropertyGroup):
    project_path: StringProperty(
        name="Path",
        description="Path to a directory which contains shiba.yml",
        subtype='DIR_PATH',
        update=_update,
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
