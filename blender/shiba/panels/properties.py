from bl_ui.utils import PresetPanel
from bpy.types import Panel
from shiba import instrumentation, operators
from shiba.render_engine import RenderEngine
from shiba.presets import AddProjectPreset


class _Panel:
    bl_context = 'render'
    bl_region_type = 'WINDOW'
    bl_space_type = 'PROPERTIES'
    COMPAT_ENGINES = {RenderEngine.bl_idname}

    @classmethod
    def poll(cls, context):
        return context.engine in cls.COMPAT_ENGINES


class SHIBA_RENDER_PT_export(_Panel, Panel):
    bl_label = "Export"

    def draw_header_preset(self, context):
        SHIBA_PT_project_presets.draw_panel_header(self.layout)

    def draw(self, context):
        layout = self.layout

        scene = context.scene
        settings = scene.shiba

        layout.prop(settings, 'export_directory', text="Directory")
        layout.prop(settings, 'export_output', text="Output")

        layout.operator(operators.build_and_export.BuildAndExportOperator.bl_idname)


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
        settings = scene.shiba

        layout.use_property_split = True
        layout.use_property_decorate = False

        layout.prop(settings, 'project_directory', text="Directory")
        layout.prop(settings, 'build_executable_on_change')

        layout.operator(operators.build_and_run.BuildAndRunOperator.bl_idname)


class SHIBA_PT_Status(_Panel, Panel):
    bl_label = "Status"

    def draw(self, context):
        layout = self.layout
        current_state = instrumentation.get_current_state()

        if current_state.server.connected:
            layout.label(text="Connected to server.")
        else:
            layout.label(text="Not connected to server.")

        if current_state.library.loaded:
            layout.label(text="Library loaded.")
        else:
            layout.label(text="Library not loaded.")
