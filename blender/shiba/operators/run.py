import bpy
from shiba import instrumentation


class RunOperator(bpy.types.Operator):
    bl_idname = 'shiba.run'
    bl_label = "Run"

    def execute(self, context):
        with instrumentation.server.get_server_connection() as server_connection:
            if server_connection:
                server_connection.send_run_command()
        return {'FINISHED'}
