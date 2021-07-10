import asyncio
from unittest import TestCase

from aioquic.asyncio.compat import _asynccontextmanager

from .utils import run


@_asynccontextmanager
async def some_context():
    await asyncio.sleep(0)
    yield
    await asyncio.sleep(0)


class AsyncioCompatTest(TestCase):
    def test_ok(self):
        async def test():
            async with some_context():
                pass

        run(test())

    def test_raise_exception(self):
        async def test():
            async with some_context():
                raise RuntimeError("some reason")

        with self.assertRaises(RuntimeError):
            run(test())

    def test_raise_exception_type(self):
        async def test():
            async with some_context():
                raise RuntimeError

        with self.assertRaises(RuntimeError):
            run(test())
