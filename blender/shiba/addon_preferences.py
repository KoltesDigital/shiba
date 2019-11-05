import bpy
from bpy.props import StringProperty


class AddonPreferences(bpy.types.AddonPreferences):
    bl_idname = __package__

    override_shiba_path: StringProperty(
        name="Override path to shiba.exe",
        description="If ever you need to use a custom build instead of the \
built-in tool. Leave empty otherwise",
        subtype='FILE_PATH',
    )

    def draw(self, context):
        layout = self.layout
        layout.prop(self, 'override_shiba_path')
