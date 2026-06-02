import multiprocessing
import sys

def max_parallelism() -> int:
    cpu_count = multiprocessing.cpu_count()
    if sys.platform == 'win32':
        # On Python 3 on Windows, using >= MAXIMUM_WAIT_OBJECTS processes
        # causes a crash in the multiprocessing module. Whilst this enum
        # can technically have any value, it is usually 64. For safety,
        # restrict manifest regeneration to 56 processes on Windows.
        #
        # See https://bugs.python.org/issue26903 and https://bugs.python.org/issue40263
        cpu_count = min(cpu_count, 56)
    return cpu_count
