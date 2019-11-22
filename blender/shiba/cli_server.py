from shiba import addon_preferences, paths
from shiba.locked_file import LockedFile
import subprocess
from threading import Lock, Thread


def _run_process_thread():
    while True:
        try:
            line = _process.stdout.readline()
            print('CLI: %s' % line.decode().rstrip())
        except ValueError:
            if _process.poll() is not None:
                break

    rc = _process.poll()
    print('CLI server exited with code %d.' % rc)


def _start_process(path):
    global _process
    global _process_thread

    print("Starting CLI server.")

    preferences = addon_preferences.get()
    _process = subprocess.Popen(
        [
            path, 'server',
            '--ip', preferences.server_ip,
            '--port', str(preferences.server_port),
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )

    _process_thread = Thread(
        target=_run_process_thread
    )
    _process_thread.start()

    print("CLI server started.")


def _end_process():
    global _process
    global _process_thread

    print("Exiting CLI server.")
    _process.terminate()

    try:
        _process.communicate(timeout=15)
    except subprocess.TimeoutExpired:
        print("Forcing CLI server to exit.")
        _process.kill()
        _process.communicate()

    _process_thread.join()
    _process_thread = None
    _process = None

    print("CLI server stopped.")


_process = None
_process_thread = None

_locked_file = LockedFile(_start_process, _end_process)
_lock = Lock()


def start():
    update_cli_path()
    with _lock:
        _locked_file.open()


def stop():
    with _lock:
        _locked_file.close()


def update_cli_path():
    with _lock:
        _locked_file.set_path(paths.cli())
