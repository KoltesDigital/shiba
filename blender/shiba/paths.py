import bpy
import json
import os

# For development purpose, change paths to use paths from the project itself.
__project_path = None

addons_to_load = os.environ.get('ADDONS_TO_LOAD')
if addons_to_load is not None:
    addons_to_load = json.loads(addons_to_load)
    for addon_to_load in addons_to_load:
        if addon_to_load['module_name'] == __package__:
            __project_path = os.path.dirname(
                os.path.dirname(addon_to_load['load_dir']))
            break

if __project_path is not None:
    __shiba_path = os.path.join(
        __project_path, 'rust', 'target', 'debug', 'shiba.exe')
else:
    __shiba_path = os.path.join(os.path.dirname(__file__), 'shiba')


def shiba():
    path = bpy.context.preferences.addons[__package__]\
        .preferences.get('override_shiba_path', None)
    if path:
        return path

    return __shiba_path
