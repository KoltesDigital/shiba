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
        if instrumentation.current_state.library.loaded:
            layout.label(text='API loaded.')
        else:
            layout.label(text='API not loaded.')


class SHIBA_PT_Uniforms(_Panel, bpy.types.Panel):
    bl_label = "Uniforms"

    def draw(self, context):
        layout = self.layout
        scene = context.scene
        render_settings = scene.shiba

        for uniform_descriptor in uniforms.get_active_uniform_descriptors():
            annotation_control = uniform_descriptor.get_annotation(
                uniforms.UniformAnnotationControlDescriptor)
            if annotation_control is not None:
                annotation_control.draw_property(
                    render_settings.uniforms, layout)
