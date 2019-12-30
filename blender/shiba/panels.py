import bpy
from shiba import instrumentation, uniforms


class _Panel:
    bl_category = "Shiba"
    bl_region_type = 'UI'
    bl_space_type = 'VIEW_3D'


class SHIBA_PT_Status(_Panel, bpy.types.Panel):
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


class SHIBA_PT_Uniforms(_Panel, bpy.types.Panel):
    bl_label = "Uniforms"

    def draw(self, context):
        layout = self.layout
        scene = context.scene
        render_settings = scene.shiba

        for uniform_descriptor in uniforms.get_active_uniform_descriptors():
            control_annotation = uniform_descriptor.get_annotation(uniforms.UniformControlAnnotationDescriptor)
            if control_annotation is not None:
                control_annotation.draw_property(render_settings.uniforms, layout)
