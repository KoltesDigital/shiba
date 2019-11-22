from itertools import filterfalse


class BrokenCallbackList:
    def __init__(self):
        self._callbacks = []

    @staticmethod
    def _call_callback_and_should_be_removed(callback):
        try:
            callback()
            return False
        except ReferenceError:
            return True

    def trigger(self):
        self._callbacks[:] = filterfalse(
            BrokenCallbackList._call_callback_and_should_be_removed,
            self._callbacks
        )

    def trigger_and_clear(self):
        for callback in self._callbacks:
            BrokenCallbackList._call_callback_and_should_be_removed(callback)
        self._callbacks.clear()

    def add(self, callback):
        self._callbacks.append(callback)

    def remove(self, callback):
        self._callbacks.remove(callback)
