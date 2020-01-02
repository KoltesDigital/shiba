import bpy
from shiba import uniforms


class _Panel:
    bl_category = "Shiba"
    bl_region_type = 'UI'
    bl_space_type = 'VIEW_3D'


class SHIBA_PT_Uniforms(_Panel, bpy.types.Panel):
    bl_label = "Uniforms"

    def draw(self, context):
        layout = self.layout
        scene = context.scene
        settings = scene.shiba

        for uniform_descriptor in uniforms.get_active_uniform_descriptors():
            control_annotation = uniform_descriptor.get_annotation(uniforms.UniformControlAnnotationDescriptor)
            if control_annotation is not None:
                control_annotation.draw_property(settings.uniforms, layout)
