# -*- coding: utf-8 -*-
"""
coroutine_tests
~~~~~~~~~~~~~~~

This file gives access to a coroutine-based test class. This allows each test
case to be defined as a pair of interacting coroutines, sending data to each
other by yielding the flow of control.

The advantage of this method is that we avoid the difficulty of using threads
in Python, as well as the pain of using sockets and events to communicate and
organise the communication. This makes the tests entirely deterministic and
makes them behave identically on all platforms, as well as ensuring they both
succeed and fail quickly.
"""
import itertools
import functools

import pytest


class CoroutineTestCase(object):
    """
    A base class for tests that use interacting coroutines.

    The run_until_complete method takes a number of coroutines as arguments.
    Each one is, in order, passed the output of the previous coroutine until
    one is exhausted. If a coroutine does not initially yield data (that is,
    its first action is to receive data), the calling code should prime it by
    using the 'server' decorator on this class.
    """
    def run_until_complete(self, *coroutines):
        """
        Executes a set of coroutines that communicate between each other. Each
        one is, in order, passed the output of the previous coroutine until
        one is exhausted. If a coroutine does not initially yield data (that
        is, its first action is to receive data), the calling code should prime
        it by using the 'server' decorator on this class.

        Once a coroutine is exhausted, the method performs a final check to
        ensure that all other coroutines are exhausted. This ensures that all
        assertions in those coroutines got executed.
        """
        looping_coroutines = itertools.cycle(coroutines)
        data = None

        for coro in looping_coroutines:
            try:
                data = coro.send(data)
            except StopIteration:
                break

        for coro in coroutines:
            try:
                next(coro)
            except StopIteration:
                continue
            else:
                pytest.fail("Coroutine %s not exhausted" % coro)

    def server(self, func):
        """
        A decorator that marks a test coroutine as a 'server' coroutine: that
        is, one whose first action is to consume data, rather than one that
        initially emits data. The effect of this decorator is simply to prime
        the coroutine.
        """
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            c = func(*args, **kwargs)
            next(c)
            return c

        return wrapper
