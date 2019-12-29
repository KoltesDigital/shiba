import bpy
from shiba import callback_lists, instrumentation, uniforms


class RenderEngine(bpy.types.RenderEngine):
    bl_idname = 'SHIBA'
    bl_label = "Shiba"

    bl_use_preview = True
    bl_use_spherical_stereo = False

    def __init__(self):
        self.__first_time_update = True
        callback_lists.viewport_update.add(self.__update_viewport)
        with instrumentation.update_state() as state:
            state.library.loaded = True
            state.server.started = True

    @staticmethod
    def _get_time(depsgraph):
        scene = depsgraph.scene
        actual_fps = scene.render.fps / scene.render.fps_base
        time = scene.frame_current / actual_fps
        return time

    @staticmethod
    def _get_view_resolution(context):
        region = context.region
        width = region.width
        height = region.height
        return width, height

    def __common_update(self, library_wrapper, depsgraph):
        if self.__first_time_update or depsgraph.id_type_updated('OBJECT'):
            library_wrapper.set_object_instances(depsgraph.object_instances)
        self.__first_time_update = False

    def update(self, data, depsgraph):
        scene = depsgraph.scene
        scene.view_settings.view_transform = 'Raw'

        with instrumentation.library.get_library_wrapper() as library_wrapper:
            if library_wrapper:
                self.__common_update(library_wrapper, depsgraph)

                time = RenderEngine._get_time(depsgraph)

                library_wrapper.update(
                    time,
                    self.resolution_x,
                    self.resolution_y,
                    self.is_preview,
                )

    def render(self, depsgraph):
        with instrumentation.library.get_library_wrapper() as library_wrapper:
            if library_wrapper:
                scene = depsgraph.scene
                time = RenderEngine._get_time(depsgraph)

                views = self.get_result().views
                for view in views:
                    self.active_view_set(view.name)

                    camera = self.camera_override or scene.camera
                    use_spherical_stereo = self.use_spherical_stereo(camera)
                    camera_matrix = self.camera_model_matrix(
                        camera, use_spherical_stereo=use_spherical_stereo)
                    projection_matrix = camera.calc_matrix_camera(
                        depsgraph,
                        x=self.render.resolution_x,
                        y=self.render.resolution_y,
                        scale_x=self.render.pixel_aspect_x,
                        scale_y=self.render.pixel_aspect_y,
                    )
                    library_wrapper.set_override_matrices(
                        camera_matrix.inverted(),
                        camera_matrix,
                        projection_matrix,
                    )

                    frame = library_wrapper.render(
                        time,
                        self.resolution_x,
                        self.resolution_y,
                        self.is_preview,
                    )

                    if frame:
                        result = self.begin_result(
                            0, 0, self.resolution_x, self.resolution_y,
                            view=view.name)
                        layer = result.layers[0].passes["Combined"]
                        layer.rect = frame
                        self.end_result(result)

    def __update_viewport(self):
        self.tag_update()
        self.tag_redraw()

    def view_update(self, context, depsgraph):
        with instrumentation.library.get_library_wrapper() as library_wrapper:
            if library_wrapper:
                self.__common_update(library_wrapper, depsgraph)

                time = RenderEngine._get_time(depsgraph)
                width, height = RenderEngine._get_view_resolution(context)
                library_wrapper.viewport_update(time, width, height)

    def view_draw(self, context, depsgraph):
        with instrumentation.library.get_library_wrapper() as library_wrapper:
            if library_wrapper:
                library_wrapper.set_override_matrices(
                    context.region_data.view_matrix,
                    context.region_data.view_matrix.inverted(),
                    context.region_data.window_matrix,
                )

                values = uniforms.get_api_uniform_values(context.scene.shiba.uniforms)
                library_wrapper.set_uniform_values(values)

                time = RenderEngine._get_time(depsgraph)
                width, height = RenderEngine._get_view_resolution(context)
                library_wrapper.viewport_render(time, width, height)


def get_panels():
    exclude_panels = {
        'VIEWLAYER_PT_filter',
        'VIEWLAYER_PT_layer_passes',
    }

    panels = []
    for panel in bpy.types.Panel.__subclasses__():
        if hasattr(panel, 'COMPAT_ENGINES') and 'BLENDER_RENDER' in panel.COMPAT_ENGINES:
            if panel.__name__ not in exclude_panels:
                panels.append(panel)

    return panels


def register():
    for panel in get_panels():
        panel.COMPAT_ENGINES.add(RenderEngine.bl_idname)


def unregister():
    for panel in get_panels():
        if RenderEngine.bl_idname in panel.COMPAT_ENGINES:
            panel.COMPAT_ENGINES.remove(RenderEngine.bl_idname)
