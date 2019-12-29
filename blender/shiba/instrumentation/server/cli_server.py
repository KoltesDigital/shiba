from shiba.instrumentation.server.desired_state import desired_state
from shiba.instrumentation.locked_file import LockedFile
import subprocess
from threading import Thread

_process = None
_process_thread = None


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

    _process = subprocess.Popen(
        [
            path,
            'server',
            '--ip', desired_state.ip,
            '--port', str(desired_state.port),
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


locked_file = LockedFile(_start_process, _end_process)
