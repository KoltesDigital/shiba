import json
import os

project_path = None
_addons_to_load = os.environ.get('ADDONS_TO_LOAD')
if _addons_to_load is not None:
    _addons_to_load = json.loads(_addons_to_load)
    for addon_to_load in _addons_to_load:
        if addon_to_load['module_name'] == __package__:
            project_path = os.path.dirname(os.path.dirname(addon_to_load['load_dir']))
            os.environ['RUST_BACKTRACE'] = '1'
            break
