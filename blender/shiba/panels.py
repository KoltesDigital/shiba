import bpy
from shiba import api


class _Panel:
    bl_category = "Shiba"
    bl_region_type = 'UI'
    bl_space_type = 'VIEW_3D'


class SHIBA_PT_Status(bpy.types.Panel, _Panel):
    bl_label = "Status"

    def draw(self, context):
        layout = self.layout
        if api.is_loaded():
            layout.label(text='API loaded.')
        else:
            layout.label(text='API not loaded.')
