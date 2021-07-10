import multiprocessing

_context = None


def get_context():
    global _context

    if _context is None:
        _context = multiprocessing.get_context("spawn")
    return _context
