import time
import timeit

# sleep can be lower than requested value in some platforms: https://bugs.python.org/issue31539
# We add padding here to compensate for that.
sleep_padding = 15.0

def sleep_at_least(sleep_in_ms):
    sleep_until = timeit.default_timer() + (sleep_in_ms / 1E3)
    time.sleep((sleep_in_ms + sleep_padding) / 1E3)
    # Check if the padding was sufficient; if not, sleep again.
    while timeit.default_timer() < sleep_until:
        time.sleep(sleep_padding / 1E3)
