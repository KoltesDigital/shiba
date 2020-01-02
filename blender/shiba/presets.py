from bl_operators.presets import AddPresetBase
from bpy.types import Operator


class AddProjectPreset(AddPresetBase, Operator):
    '''Add a Project Preset'''
    bl_idname = "render.shiba_project_preset_add"
    bl_label = "Add Project Preset"
    preset_menu = 'SHIBA_PT_project_presets'

    preset_defines = [
        "settings = bpy.context.scene.shiba"
    ]

    preset_values = [
        "settings.build_executable_on_change",
        "settings.project_directory",
    ]

    preset_subdir = "shiba/project"
