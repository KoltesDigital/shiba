import bpy
from bpy.props import IntProperty, StringProperty


class AddonPreferences(bpy.types.AddonPreferences):
    bl_idname = __package__

    notification_size: IntProperty(
        name="Notification size",
        default=50,
        min=0,
    )

    override_shiba_path: StringProperty(
        name="Override path to shiba.exe",
        description="If ever you need to use a custom build instead of the \
built-in tool. Leave empty otherwise",
        subtype='FILE_PATH',
    )

    def draw(self, context):
        layout = self.layout
        layout.prop(self, 'notification_size')
        layout.prop(self, 'override_shiba_path')


def get(key, default):
    return bpy.context.preferences.addons[__package__]\
        .preferences.get(key, default)
