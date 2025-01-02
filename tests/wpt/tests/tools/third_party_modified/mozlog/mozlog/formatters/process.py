# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import signal

# a dict cache of signal number -> signal name
_SIG_NAME = None


def strsig(n):
    """
    Translate a process signal identifier to a human readable string.
    """
    global _SIG_NAME

    if _SIG_NAME is None:
        # cache signal names
        _SIG_NAME = {}
        for k in dir(signal):
            if (
                k.startswith("SIG") and
                not k.startswith("SIG_") and
                k != "SIGCLD" and
                k != "SIGPOLL"
            ):

                _SIG_NAME[getattr(signal, k)] = k

        # Realtime signals mostly have no names
        if hasattr(signal, "SIGRTMIN") and hasattr(signal, "SIGRTMAX"):
            for r in range(signal.SIGRTMIN + 1, signal.SIGRTMAX + 1):
                _SIG_NAME[r] = "SIGRTMIN+" + str(r - signal.SIGRTMIN)

    if n < 0 or n >= signal.NSIG:
        return "out-of-range signal, number %s" % n
    try:
        return _SIG_NAME[n]
    except KeyError:
        return "unrecognized signal, number %s" % n


def strstatus(status):
    """
    Returns a human readable string of a process exit code, as returned
    by the subprocess module.
    """
    # 'status' is the exit status
    if os.name != "posix":
        # Windows error codes are easier to look up if printed in hexadecimal
        if status < 0:
            status += 2 ** 32
        return "exit %x" % status
    elif status >= 0:
        return "exit %d" % status
    else:
        return "killed by %s" % strsig(-status)
