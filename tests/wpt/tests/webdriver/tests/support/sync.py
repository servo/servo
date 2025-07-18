import asyncio
import collections
import inspect
import sys
import time

from webdriver import error


DEFAULT_TIMEOUT = 5
DEFAULT_INTERVAL = 0.1


class Poll(object):
    """
    An explicit conditional utility primitive for polling until a
    condition evaluates to something truthy.

    A `Poll` instance defines the maximum amount of time to wait
    for a condition, as well as the frequency with which to check
    the condition.  Furthermore, the user may configure the wait
    to ignore specific types of exceptions whilst waiting, such as
    `error.NoSuchElementException` when searching for an element
    on the page.
    """

    def __init__(
        self,
        session,
        timeout=DEFAULT_TIMEOUT,
        interval=DEFAULT_INTERVAL,
        raises=error.TimeoutException,
        message=None,
        ignored_exceptions=None,
        clock=time,
    ):
        """
        Configure the poller to have a custom timeout, interval,
        and list of ignored exceptions.  Optionally a different time
        implementation than the one provided by the standard library
        (`time`) can also be provided.

        Sample usage::

            # Wait 30 seconds for window to open,
            # checking for its presence once every 5 seconds.
            from support.sync import Poll
            wait = Poll(session, timeout=30, interval=5,
                        ignored_exceptions=error.NoSuchWindowException)
            window = wait.until(lambda s: s.switch_to_window(42))

        :param session: The input value to be provided to conditions,
            usually a `webdriver.Session` instance.

        :param timeout: How long to wait for the evaluated condition
            to become true.

        :param interval: How often the condition should be evaluated.
            In reality the interval may be greater as the cost of
            evaluating the condition function. If that is not the case the
            interval for the next condition function call is shortend to keep
            the original interval sequence as best as possible.

        :param raises: Optional exception to raise when poll elapses.
            If not used, an `error.TimeoutException` is raised.
            If it is `None`, no exception is raised on the poll elapsing.

        :param message: An optional fallback message to include in `raises`'s
            message if the `until` condition times out and no assertion failure
            or other exception message from calling the condition callback is
            available.

        :param ignored_exceptions: Ignore specific types of exceptions
            whilst waiting for the condition.  Any exceptions not in this list
            will be allowed to propagate, terminating the wait.

        :param clock: Allows overriding the use of the runtime's
            default time library.
        """
        self.session = session
        self.timeout = timeout
        self.interval = interval
        self.exc_cls = raises
        self.exc_msg = message
        self.clock = clock

        exceptions = [AssertionError]
        if ignored_exceptions is not None:
            if isinstance(ignored_exceptions, collections.abc.Iterable):
                exceptions.extend(iter(ignored_exceptions))
            else:
                exceptions.append(ignored_exceptions)
        self.exceptions = tuple(set(exceptions))

    def until(self, condition):
        """
        This will repeatedly evaluate `condition` in anticipation
        for a truthy return value, or the timeout to expire.

        A condition that returns `None` or does not evaluate to
        true will fully elapse its timeout before raising, unless
        the `raises` keyword argument is `None`, in which case the
        condition's return value is propagated unconditionally.

        If an exception is raised in `condition` and it's not ignored,
        this function will raise immediately.  If the exception is
        ignored it will be swallowed and polling will resume until
        either the condition meets the return requirements or the
        timeout duration is reached.

        :param condition: A callable function whose return value will
            be returned by this function. Use assert statements within this
            callback to see more detailed error messages.
        """
        start = self.clock.time()
        end = start + self.timeout

        while not self.clock.time() >= end:
            condition_msg = result = traceback = None

            try:
                next = self.clock.time() + self.interval
                result = condition(self.session)
            except (KeyboardInterrupt, SystemExit):
                raise
            except self.exceptions:
                _, condition_msg, traceback = sys.exc_info()

            # re-adjust the interval depending on how long
            # the callback took to evaluate the condition
            interval_new = max(next - self.clock.time(), 0)

            if result is None:  # assertions were used, so no return value
                if not traceback:  # assertions in the condition all passed, we are done
                    return
            else:
                if result:  # condition returned True, we are done
                    return result

            self.clock.sleep(interval_new)

        if self.exc_cls is not None:
            elapsed = round((self.clock.time() - start), 1)
            message = f"Timed out after {elapsed} seconds"
            if condition_msg or self.exc_msg:
                message += f" with message: {condition_msg or self.exc_msg}"
            raise self.exc_cls(message=message).with_traceback(traceback)
        else:
            return result


class AsyncPoll(object):
    """
    An explicit conditional utility primitive for asynchronously polling
    until a condition evaluates to something truthy.

    A `Poll` instance defines the maximum amount of time to wait
    for a condition, as well as the frequency with which to check
    the condition.  Furthermore, the user may configure the wait
    to ignore specific types of exceptions whilst waiting, such as
    `error.NoSuchElementException` when searching for an element
    on the page.
    """

    def __init__(
        self,
        session,
        timeout=DEFAULT_TIMEOUT,
        interval=DEFAULT_INTERVAL,
        raises=error.TimeoutException,
        message=None,
        ignored_exceptions=None,
        clock=None,
    ):
        """
        Configure the poller to have a custom timeout, interval,
        and list of ignored exceptions.  Optionally a different time
        implementation than the one provided by the event loop
        (`asyncio.get_event_loop()`) can also be provided.

        Sample usage::

            # Wait 30 seconds for window to open,
            # checking for its presence once every 5 seconds.
            from support.sync import AsyncPoll
            wait = AsyncPoll(session, timeout=30, interval=5,
                        ignored_exceptions=error.NoSuchWindowException)
            window = await wait.until(lambda s: s.switch_to_window(42))

        :param session: The input value to be provided to conditions,
            usually a `webdriver.Session` instance.

        :param timeout: How long to wait for the evaluated condition
            to become true.

        :param interval: How often the condition should be evaluated.
            In reality the interval may be greater as the cost of
            evaluating the condition function. If that is not the case the
            interval for the next condition function call is shortend to keep
            the original interval sequence as best as possible.

        :param raises: Optional exception to raise when poll elapses.
            If not used, an `error.TimeoutException` is raised.
            If it is `None`, no exception is raised on the poll elapsing.

        :param message: An optional fallback message to include in `raises`'s
            message if the `until` condition times out and no assertion failure
            or other exception message from calling the condition callback is
            available.

        :param ignored_exceptions: Ignore specific types of exceptions
            whilst waiting for the condition.  Any exceptions not in this list
            will be allowed to propagate, terminating the wait.

        :param clock: Allows overriding the use of the asyncio.get_event_loop()
        default time implementation.
        """
        self.session = session
        self.timeout = timeout
        self.interval = interval
        self.exc_cls = raises
        self.exc_msg = message
        self.clock = clock if clock is not None else asyncio.get_event_loop()

        exceptions = [AssertionError]
        if ignored_exceptions is not None:
            if isinstance(ignored_exceptions, collections.abc.Iterable):
                exceptions.extend(iter(ignored_exceptions))
            else:
                exceptions.append(ignored_exceptions)
        self.exceptions = tuple(set(exceptions))

    async def until(self, condition, *args, **kwargs):
        """
        This will repeatedly evaluate `condition` in anticipation
        for a truthy return value, or the timeout to expire.

        A condition that returns `None` or does not evaluate to
        true will fully elapse its timeout before raising, unless
        the `raises` keyword argument is `None`, in which case the
        condition's return value is propagated unconditionally.

        If an exception is raised in `condition` and it's not ignored,
        this function will raise immediately.  If the exception is
        ignored it will be swallowed and polling will resume until
        either the condition meets the return requirements or the
        timeout duration is reached.

        :param condition: A callable function whose return value will
            be returned by this function. Use assert statements within this
            callback to see more detailed error messages.
        """

        async def poll():
            start = self.clock.time()
            end = start + self.timeout

            while not self.clock.time() >= end:
                condition_msg = result = traceback = None

                next = self.clock.time() + self.interval

                try:
                    _result = condition(self.session, *args, **kwargs)
                    result = await _result if inspect.isawaitable(_result) else _result
                except (KeyboardInterrupt, SystemExit):
                    raise
                except self.exceptions:
                    _, condition_msg, traceback = sys.exc_info()

                # re-adjust the interval depending on how long
                # the callback took to evaluate the condition
                interval_new = max(next - self.clock.time(), 0)

                if result is None:  # assertions were used, so no return value
                    if not traceback:  # assertions in the condition all passed, we are done
                        return
                else:
                    if result:  # condition returned True, we are done
                        return result

                await asyncio.sleep(interval_new)

            if self.exc_cls is not None:
                elapsed = round((self.clock.time() - start), 1)
                message = f"Timed out after {elapsed} seconds"
                if condition_msg or self.exc_msg:
                    message += f" with message: {condition_msg or self.exc_msg}"
                raise self.exc_cls(message=message).with_traceback(traceback)
            else:
                return result

        return await poll()
