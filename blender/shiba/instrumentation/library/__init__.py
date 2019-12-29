import _ctypes
from contextlib import contextmanager
import ctypes
from dataclasses import dataclass
import os.path
from shiba import callback_lists, uniforms
from shiba.instrumentation.error import StateMatchFailure
from shiba.instrumentation.library.desired_state import desired_state
from shiba.instrumentation.library.library_wrapper import LibraryWrapper
from shiba.instrumentation.locked_file import LockedFile
from threading import Lock


@dataclass
class _CurrentState:
    @property
    def loaded(self):
        return _locked_file.is_opened

    @property
    def path(self):
        return _locked_file.path


current_state = _CurrentState()


_library = None
_library_wrapper = None


def _load_library(path):
    global _library
    global _library_wrapper

    print("Loading library.")

    _library = ctypes.CDLL(path)

    _library_wrapper = LibraryWrapper(_library)
    count, descriptors = _library_wrapper.get_active_uniform_descriptors()
    uniforms.set_api_active_uniform_descriptors(count, descriptors)

    callback_lists.viewport_update.trigger()

    print("Library loaded.")


def _close_library():
    global _library
    global _library_wrapper

    print("Unloading library.")

    _ctypes.FreeLibrary(_library._handle)
    _library = None

    _library_wrapper = None

    print("Library unloaded.")


_lock = Lock()
_locked_file = LockedFile(_load_library, _close_library)


def match_states():
    with _lock:
        if _locked_file.path != desired_state.path:
            # Make sure indirect DLL dependencies are found.
            parent_path = os.path.dirname(desired_state.path)
            if os.environ["PATH"].find(parent_path) == -1:
                os.environ["PATH"] += ";" + parent_path

            _locked_file.set_path(desired_state.path)

        if _locked_file.path:
            if not _locked_file.is_opened and desired_state.loaded:
                result = _locked_file.open()
                if not result:
                    raise StateMatchFailure()

            if _locked_file.is_opened and not desired_state.loaded:
                _locked_file.close()


@contextmanager
def get_library_wrapper():
    with _lock:
        yield _library_wrapper
