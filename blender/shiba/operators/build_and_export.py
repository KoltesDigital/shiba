import bpy
from shiba import instrumentation, path


class BuildAndExportOperator(bpy.types.Operator):
    bl_idname = 'shiba.build_and_export'
    bl_label = "Build & Export"

    def execute(self, context):
        settings = context.scene.shiba
        directory = path.realpath(settings.export_directory)
        output = settings.export_output

        with instrumentation.update_state() as state:
            state.library.loaded = True
            state.server.connected = True

        with instrumentation.server.get_server_connection() as server_connection:
            if server_connection:
                server_connection.send_build_command('executable')
                server_connection.send_export_command(directory, output, 'executable')

        return {'FINISHED'}
