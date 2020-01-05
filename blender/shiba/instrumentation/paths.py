import os
from shiba import development

# For development purpose, change paths to use paths from the project itself.
if development.project_path is not None:
    _shiba_cli_path = os.path.join(development.project_path, 'rust', 'target', 'debug', 'shiba-cli.exe')
else:
    _shiba_cli_path = os.path.join(os.path.dirname(__file__), '..', 'shiba-cli.exe')


def cli():
    return _shiba_cli_path
