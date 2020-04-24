from contextlib import ContextDecorator
from functools import wraps

try:
    from contextlib import asynccontextmanager
except ImportError:
    asynccontextmanager = None


class _AsyncGeneratorContextManager(ContextDecorator):
    def __init__(self, func, args, kwds):
        self.gen = func(*args, **kwds)
        self.func, self.args, self.kwds = func, args, kwds
        self.__doc__ = func.__doc__

    async def __aenter__(self):
        return await self.gen.__anext__()

    async def __aexit__(self, typ, value, traceback):
        if typ is not None:
            await self.gen.athrow(typ, value, traceback)


def _asynccontextmanager(func):
    @wraps(func)
    def helper(*args, **kwds):
        return _AsyncGeneratorContextManager(func, args, kwds)

    return helper


if asynccontextmanager is None:
    asynccontextmanager = _asynccontextmanager
