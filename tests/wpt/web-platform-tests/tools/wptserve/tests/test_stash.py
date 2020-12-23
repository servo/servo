import multiprocessing
import threading
import sys

from multiprocessing.managers import BaseManager

import pytest
from six import PY3

Stash = pytest.importorskip("wptserve.stash").Stash

@pytest.fixture()
def add_cleanup():
    fns = []

    def add(fn):
        fns.append(fn)

    yield add

    for fn in fns:
        fn()


def run(process_queue, request_lock, response_lock):
    """Create two Stash instances in parallel threads. Use the provided locks
    to ensure the first thread is actively establishing an interprocess
    communication channel at the moment the second thread executes."""

    def target(thread_queue):
        stash = Stash("/", ("localhost", 4543), b"some key")

        # The `lock` property of the Stash instance should always be set
        # immediately following initialization. These values are asserted in
        # the active test.
        thread_queue.put(stash.lock is None)

    thread_queue = multiprocessing.Queue()
    first = threading.Thread(target=target, args=(thread_queue,))
    second = threading.Thread(target=target, args=(thread_queue,))

    request_lock.acquire()
    response_lock.acquire()
    first.start()

    request_lock.acquire()

    # At this moment, the `first` thread is waiting for a proxied object.
    # Create a second thread in order to inspect the behavior of the Stash
    # constructor at this moment.

    second.start()

    # Allow the `first` thread to proceed

    response_lock.release()

    # Wait for both threads to complete and report their stateto the test
    process_queue.put(thread_queue.get())
    process_queue.put(thread_queue.get())


class SlowLock(BaseManager):
    # This can only be used in test_delayed_lock since that test modifies the
    # class body, but it has to be a global for multiprocessing
    pass


@pytest.mark.xfail(sys.platform == "win32" or
                   PY3 and multiprocessing.get_start_method() == "spawn",
                   reason="https://github.com/web-platform-tests/wpt/issues/16938")
def test_delayed_lock(add_cleanup):
    """Ensure that delays in proxied Lock retrieval do not interfere with
    initialization in parallel threads."""

    request_lock = multiprocessing.Lock()
    response_lock = multiprocessing.Lock()

    queue = multiprocessing.Queue()

    def mutex_lock_request():
        """This request handler allows the caller to delay execution of a
        thread which has requested a proxied representation of the `lock`
        property, simulating a "slow" interprocess communication channel."""

        request_lock.release()
        response_lock.acquire()
        return threading.Lock()

    SlowLock.register("get_dict", callable=lambda: {})
    SlowLock.register("Lock", callable=mutex_lock_request)

    slowlock = SlowLock(("localhost", 4543), b"some key")
    slowlock.start()
    add_cleanup(lambda: slowlock.shutdown())

    parallel = multiprocessing.Process(target=run,
                                       args=(queue, request_lock, response_lock))
    parallel.start()
    add_cleanup(lambda: parallel.terminate())

    assert [queue.get(), queue.get()] == [False, False], (
        "both instances had valid locks")


class SlowDict(BaseManager):
    # This can only be used in test_delayed_dict since that test modifies the
    # class body, but it has to be a global for multiprocessing
    pass


@pytest.mark.xfail(sys.platform == "win32" or
                   PY3 and multiprocessing.get_start_method() == "spawn",
                   reason="https://github.com/web-platform-tests/wpt/issues/16938")
def test_delayed_dict(add_cleanup):
    """Ensure that delays in proxied `dict` retrieval do not interfere with
    initialization in parallel threads."""

    request_lock = multiprocessing.Lock()
    response_lock = multiprocessing.Lock()

    queue = multiprocessing.Queue()

    # This request handler allows the caller to delay execution of a thread
    # which has requested a proxied representation of the "get_dict" property.
    def mutex_dict_request():
        """This request handler allows the caller to delay execution of a
        thread which has requested a proxied representation of the `get_dict`
        property, simulating a "slow" interprocess communication channel."""
        request_lock.release()
        response_lock.acquire()
        return {}

    SlowDict.register("get_dict", callable=mutex_dict_request)
    SlowDict.register("Lock", callable=lambda: threading.Lock())

    slowdict = SlowDict(("localhost", 4543), b"some key")
    slowdict.start()
    add_cleanup(lambda: slowdict.shutdown())

    parallel = multiprocessing.Process(target=run,
                                       args=(queue, request_lock, response_lock))
    parallel.start()
    add_cleanup(lambda: parallel.terminate())

    assert [queue.get(), queue.get()] == [False, False], (
        "both instances had valid locks")
