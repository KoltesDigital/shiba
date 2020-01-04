from shiba import instrumentation, notifications, uniforms
from shiba.notifications import Notification
from threading import Lock

_building_lock = Lock()
_building_count = 0


def _handle_build_ended_event(obj):
    global _building_count
    global _building_notification

    with _building_lock:
        _building_count -= 1
        if _building_count == 0:
            notifications.remove(_building_notification)

    if obj['successful']:
        message = "Build '%s' succeeded, duration: %fs." % (obj['target'], obj['duration'])
    else:
        message = "Build '%s' failed." % obj['target']

    print(message)
    notifications.add(Notification(message, 3))


def _handle_build_started_event(obj):
    global _building_count
    global _building_notification

    with _building_lock:
        if _building_count == 0:
            _building_notification = Notification("Building...")
            notifications.add(_building_notification)
        _building_count += 1


def _handle_error_event(obj):
    print("Server error: %s" % obj['message'])
    notifications.add(Notification("Error, see system console.", 5))


def _handle_executable_compiled_event(obj):
    print("Executable compiled at %s." % obj['path'])
    size = obj['size']
    notifications.add(Notification("Executable size: %d." % size, 5))


def _handle_exported_event(obj):
    print("Exported at %s." % obj['path'])
    notifications.add(Notification("Exported.", 5))


def _handle_library_compiled_event(obj):
    print("Library compiled at %s." % obj['path'])
    with instrumentation.update_state() as state:
        state.library.path = obj['path']


def _handle_run_event(obj):
    message = "Run duration: %fs." % obj['duration']
    notifications.add(Notification(message, 3))


def _handle_shader_set_provided_event(obj):
    print("Shader mapping for target '%s':" % obj['target'])
    for i, shader_source in enumerate(obj['sources']):
        print("  #%d: %s" % (i, shader_source['name']))

    if obj['target'] == 'library':
        with instrumentation.library.get_library_wrapper() as library_wrapper:
            if library_wrapper:
                library_wrapper.set_shader_sources(obj['sources'])
        uniforms.set_shader_variables(obj['variables'])


EVENT_HANDLERS = {
    'build-ended': _handle_build_ended_event,
    'build-started': _handle_build_started_event,
    'error': _handle_error_event,
    'executable-compiled': _handle_executable_compiled_event,
    'exported': _handle_exported_event,
    'library-compiled': _handle_library_compiled_event,
    'run': _handle_run_event,
    'shader-set-provided': _handle_shader_set_provided_event,
}
