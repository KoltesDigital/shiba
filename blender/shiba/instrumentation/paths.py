import json
import os

# Should be 'shiba'.
module_name = __name__.split('.')[0]

# For development purpose, change paths to use paths from the project itself.
_project_path = None

addons_to_load = os.environ.get('ADDONS_TO_LOAD')
if addons_to_load is not None:
    addons_to_load = json.loads(addons_to_load)
    for addon_to_load in addons_to_load:
        if addon_to_load['module_name'] == module_name:
            _project_path = os.path.dirname(
                os.path.dirname(addon_to_load['load_dir']))
            break

if _project_path is not None:
    _shiba_cli_path = os.path.join(_project_path, 'rust', 'target', 'debug', 'shiba-cli.exe')
else:
    _shiba_cli_path = os.path.join(os.path.dirname(__file__), 'shiba-cli.exe')


def cli():
    return _shiba_cli_path
