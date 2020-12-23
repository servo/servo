import multiprocessing

import six

_context = None


class MpContext(object):
    def __getattr__(self, name):
        return getattr(multiprocessing, name)


def get_context():
    global _context

    if _context is None:
        if six.PY2:
            _context = MpContext()
        else:
            _context = multiprocessing.get_context("spawn")
    return _context
