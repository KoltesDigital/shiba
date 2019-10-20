import bpy
from shiba.api import API

api = None


class RenderEngine(bpy.types.RenderEngine):
    bl_idname = 'SHIBA'
    bl_label = "Shiba"

    bl_use_preview = True

    def __init__(self):
        global api
        if not api:
            api = API()
            api.load()

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

    def update(self, data, depsgraph):
        scene = depsgraph.scene
        scene.view_settings.view_transform = 'Raw'

        time = RenderEngine._get_time(depsgraph)

        api.update(
            time,
            self.resolution_x,
            self.resolution_y,
            self.is_preview,
        )

    def render(self, depsgraph):
        time = RenderEngine._get_time(depsgraph)

        frame = api.render(
            time,
            self.resolution_x,
            self.resolution_y,
            self.is_preview,
        )

        if frame:
            result = self.begin_result(
                0, 0, self.resolution_x, self.resolution_y)
            layer = result.layers[0].passes["Combined"]
            layer.rect = frame
            self.end_result(result)

    def view_update(self, context, depsgraph):
        time = RenderEngine._get_time(depsgraph)
        width, height = RenderEngine._get_view_resolution(context)
        api.viewport_update(time, width, height)

    def view_draw(self, context, depsgraph):
        time = RenderEngine._get_time(depsgraph)
        width, height = RenderEngine._get_view_resolution(context)
        api.viewport_render(time, width, height)


def get_panels():
    exclude_panels = {
        'VIEWLAYER_PT_filter',
        'VIEWLAYER_PT_layer_passes',
    }

    panels = []
    for panel in bpy.types.Panel.__subclasses__():
        if hasattr(panel, 'COMPAT_ENGINES') \
                and 'BLENDER_RENDER' in panel.COMPAT_ENGINES:
            if panel.__name__ not in exclude_panels:
                panels.append(panel)

    return panels


def register():
    for panel in get_panels():
        panel.COMPAT_ENGINES.add(RenderEngine.bl_idname)


def unregister():
    global api
    if api:
        api.unload()
        api = None

    for panel in get_panels():
        if RenderEngine.bl_idname in panel.COMPAT_ENGINES:
            panel.COMPAT_ENGINES.remove(RenderEngine.bl_idname)
