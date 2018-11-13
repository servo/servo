import sys
import time


class TimeoutException(Exception):
    pass


def wait(session, condition, message,
         interval=0.1, timeout=5, ignored_exceptions=Exception):
    """ Poll a condition until it's true or the timeout ellapses.

    :param session: WebDriver session to use with `condition`
    :param condition: function that accepts a WebDriver session and returns a boolean
    :param message: failure description to display in case the timeout is reached
    :param interval: seconds between each call to `condition`. Default: 0.1
    :param timeout: seconds until we stop polling. Default: 5
    :param ignored_exceptions: Exceptions that are expected and can be ignored.
        Default: Exception
    """

    start = time.time()
    end = start + timeout

    while not (time.time() >= end):
        next_step = time.time() + interval
        try:
            success = condition(session)
        except ignored_exceptions:
            last_exc = sys.exc_info()[0]
            success = False
        next_interval = max(next_step - time.time(), 0)
        if not success:
            time.sleep(next_interval)
            continue
        return success

    raise TimeoutException("Timed out after %d seconds: %s" % (timeout, message))
