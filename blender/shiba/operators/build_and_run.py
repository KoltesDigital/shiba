import bpy
from shiba import instrumentation


class BuildAndRunOperator(bpy.types.Operator):
    bl_idname = 'shiba.build_and_run'
    bl_label = "Build & Run"

    def execute(self, context):
        with instrumentation.server.get_server_connection() as server_connection:
            if server_connection:
                server_connection.send_build_command('executable')
                server_connection.send_run_command()
        return {'FINISHED'}
