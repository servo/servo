"""Defines and sets a custom event loop policy"""
import asyncio
from asyncio import DefaultEventLoopPolicy, SelectorEventLoop


class TestEventLoop(SelectorEventLoop):
    pass


class TestEventLoopPolicy(DefaultEventLoopPolicy):
    def new_event_loop(self):
        return TestEventLoop()


# This statement represents a code which sets a custom event loop policy
asyncio.set_event_loop_policy(TestEventLoopPolicy())
