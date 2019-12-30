import blf
import bpy
from threading import Lock, Thread
from shiba import addon_preferences, callback_lists
import time


class Notification:
    def __init__(self, message, duration=None):
        self.__message = message
        self.__duration = duration

    def __str__(self):
        return self.__message

    @property
    def duration(self):
        return self.__duration


_notifications = None
_notifications_lock = None


def _draw():
    preferences = addon_preferences.get()
    if not preferences:
        return

    font_id = 0
    font_size = preferences.server_notification_size
    padding = 20

    blf.color(font_id, 1, 1, 1, 1)
    blf.enable(font_id, blf.SHADOW)
    blf.shadow(font_id, 0, 0, 0, 0, 1)
    blf.shadow_offset(font_id, 4, 4)

    y = padding
    blf.size(font_id, font_size, 72)
    with _notifications_lock:
        for notification in _notifications:
            blf.position(font_id, padding, y, 0)
            blf.draw(font_id, str(notification))
            y += font_size + padding


def add(notification):
    with _notifications_lock:
        _notifications.append(notification)
    callback_lists.viewport_update.trigger()

    if notification.duration is not None:
        def _run_thread_wait_and_remove_notification():
            time.sleep(notification.duration)
            remove(notification)

        thread = Thread(target=_run_thread_wait_and_remove_notification)
        thread.start()


def remove(notification):
    with _notifications_lock:
        _notifications.remove(notification)
    callback_lists.viewport_update.trigger()


def register():
    global _notifications
    global _notifications_lock
    global _draw_handler

    _notifications = []
    _notifications_lock = Lock()

    _draw_handler = bpy.types.SpaceView3D.draw_handler_add(
        _draw, (), 'WINDOW', 'POST_PIXEL')


def unregister():
    bpy.types.SpaceView3D.draw_handler_remove(
        _draw_handler, 'WINDOW')
