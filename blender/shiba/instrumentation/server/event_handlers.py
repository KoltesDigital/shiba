from shiba import instrumentation, notifications, uniforms
from shiba.notifications import Notification
from threading import Lock

_building_lock = Lock()
_building_count = 0


def _handle_event_build_ended(obj):
    global _building_count
    global _building_notification

    with _building_lock:
        _building_count -= 1
        if _building_count == 0:
            notifications.remove(_building_notification)

    if obj['successful']:
        str = "Build '%s' succeeded, duration: %fs." % (obj['target'], obj['duration'])
    else:
        str = "Build '%s' failed." % obj['target']

    print(str)
    notifications.add(Notification(str, 3))


def _handle_event_build_started(obj):
    global _building_count
    global _building_notification

    with _building_lock:
        if _building_count == 0:
            _building_notification = Notification("Building...")
            notifications.add(_building_notification)
        _building_count += 1


def _handle_event_executable_compiled(obj):
    print("Executable compiled at %s." % obj['path'])
    size = obj['size']
    notifications.add(Notification("Executable size: %d." % size, 5))


def _handle_event_error(obj):
    print("Server error: %s" % obj['message'])


def _handle_event_library_compiled(obj):
    print("Library compiled at %s." % obj['path'])
    with instrumentation.update_state() as state:
        state.library.path = obj['path']


def _handle_event_shader_provided(obj):
    if obj['target'] == 'library':
        with instrumentation.library.get_library_wrapper() as library_wrapper:
            if library_wrapper:
                library_wrapper.set_shader_passes(obj['passes'])
        uniforms.set_shader_variables(obj['variables'])


event_handlers = {
    'build-ended': _handle_event_build_ended,
    'build-started': _handle_event_build_started,
    'executable-compiled': _handle_event_executable_compiled,
    'error': _handle_event_error,
    'library-compiled': _handle_event_library_compiled,
    'shader-provided': _handle_event_shader_provided,
}
