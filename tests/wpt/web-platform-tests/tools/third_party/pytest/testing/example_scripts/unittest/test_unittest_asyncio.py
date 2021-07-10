from typing import List
from unittest import IsolatedAsyncioTestCase  # type: ignore


teardowns = []  # type: List[None]


class AsyncArguments(IsolatedAsyncioTestCase):
    async def asyncTearDown(self):
        teardowns.append(None)

    async def test_something_async(self):
        async def addition(x, y):
            return x + y

        self.assertEqual(await addition(2, 2), 4)

    async def test_something_async_fails(self):
        async def addition(x, y):
            return x + y

        self.assertEqual(await addition(2, 2), 3)

    def test_teardowns(self):
        assert len(teardowns) == 2
