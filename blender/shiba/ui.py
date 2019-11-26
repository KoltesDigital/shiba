from bl_ui.utils import PresetPanel
from bpy.types import Panel
from shiba.render_engine import RenderEngine
from shiba.presets import AddProjectPreset


class _Panel:
    bl_space_type = 'PROPERTIES'
    bl_region_type = 'WINDOW'
    bl_context = 'render'
    COMPAT_ENGINES = {RenderEngine.bl_idname}

    @classmethod
    def poll(cls, context):
        return context.engine in cls.COMPAT_ENGINES


class SHIBA_PT_project_presets(PresetPanel, Panel):
    bl_label = "Project Presets"
    preset_subdir = AddProjectPreset.preset_subdir
    preset_operator = "script.execute_preset"
    preset_add_operator = AddProjectPreset.bl_idname
    COMPAT_ENGINES = {RenderEngine.bl_idname}


class SHIBA_RENDER_PT_project(_Panel, Panel):
    bl_label = "Project"

    def draw_header_preset(self, context):
        SHIBA_PT_project_presets.draw_panel_header(self.layout)

    def draw(self, context):
        layout = self.layout

        scene = context.scene
        render_settings = scene.shiba

        layout.use_property_split = True
        layout.use_property_decorate = False

        layout.prop(render_settings, 'project_directory')
        layout.prop(render_settings, 'build_executable')
