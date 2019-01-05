import time

# sleep can be lower than requested value in some platforms: https://bugs.python.org/issue31539
# We add padding here to compensate for that.
sleep_padding = 15.0

def sleep_at_least(sleep_in_ms):
    time.sleep((sleep_in_ms + sleep_padding) / 1E3);
