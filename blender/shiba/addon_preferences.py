import bpy
from bpy.props import EnumProperty, IntProperty, StringProperty
from shiba import instrumentation


def _update_instrumentation(_self, _context):
    instrumentation.update()


class AddonPreferences(bpy.types.AddonPreferences):
    bl_idname = __package__

    server_custom_cli_path: StringProperty(
        name="Path to custom CLI",
        description="Full path including the executable filename",
        subtype='FILE_PATH',
        update=_update_instrumentation,
    )

    server_ip: StringProperty(
        name="Server IP",
        description="Both for internal and external servers",
        default="127.0.0.1",
        update=_update_instrumentation,
    )

    server_location: EnumProperty(
        name="Server location",
        items=[
            (
                'BUILT_IN_CLI',
                "Built-in CLI",
                "Use built-in CLI to start the server"
            ),
            (
                'CUSTOM_CLI',
                "Custom CLI",
                "Use a custom CLI to start the server"
            ),
            (
                'EXTERNAL',
                "External",
                "Connect to an external server which is already started"
            ),
        ],
        default='BUILT_IN_CLI',
        update=_update_instrumentation,
    )

    server_notification_size: IntProperty(
        name="Server notification size",
        default=50,
        min=0,
    )

    server_port: IntProperty(
        name="Server port",
        description="Both for internal and external servers",
        default=5184,
        min=0,
        update=_update_instrumentation,
    )

    def draw(self, context):
        preferences = get()
        layout = self.layout
        layout.prop(self, 'server_location')
        if preferences.server_location == 'CUSTOM_CLI':
            layout.prop(self, 'server_custom_cli_path')
        layout.prop(self, 'server_ip')
        layout.prop(self, 'server_port')

        layout.prop(self, 'server_notification_size')


def get():
    addon = bpy.context.preferences.addons.get(__package__, None)
    preferences = addon.preferences if addon else None
    return preferences
