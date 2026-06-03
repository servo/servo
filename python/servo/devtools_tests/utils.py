# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import annotations
from collections import Counter
from concurrent.futures import Future
from dataclasses import dataclass
from typing import Any, Iterable, Optional, TypeVar

from geckordp.actors.events import Events
from geckordp.actors.node import NodeActor
from geckordp.actors.resources import Resources
from geckordp.actors.root import RootActor
from geckordp.actors.descriptors.tab import TabActor
from geckordp.actors.watcher import WatcherActor
from geckordp.rdp_client import RDPClient

# Set this to true to log requests in the internal web servers.
LOG_REQUESTS = False
# The devtools server will be served at 6000.
DEVTOOLS_PORT = 6000
# Other web servers.
WEB_SERVERS = [10000, 10001]
SERVER_ADDRESS = "127.0.0.1"


@dataclass
class Devtools:
    client: RDPClient
    tab: TabActor
    watcher: WatcherActor
    targets: list
    exited: bool = False

    def connect(*, expected_targets: int = 1) -> Devtools:
        """
        Connect to the Servo devtools server.
        You should use a `with` statement to ensure we disconnect unconditionally.
        """
        client = RDPClient()
        client.connect(SERVER_ADDRESS, DEVTOOLS_PORT)
        root = RootActor(client)
        tabs = root.list_tabs()
        tab_dict = tabs[0]
        tab = TabActor(client, tab_dict["actor"])
        watcher = tab.get_watcher()
        watcher = WatcherActor(client, watcher["actor"])

        done = Future()
        targets = []

        def on_target(data):
            try:
                targets.append(data["target"])
                if len(targets) == expected_targets:
                    done.set_result(None)
            except Exception as e:
                # Raising here does nothing, for some reason.
                # Send the exception back so it can be raised.
                done.set_result(e)

        client.add_event_listener(
            watcher.actor_id,
            Events.Watcher.TARGET_AVAILABLE_FORM,
            on_target,
        )
        watcher.watch_targets(WatcherActor.Targets.FRAME)
        watcher.watch_targets(WatcherActor.Targets.WORKER)

        result: Optional[Exception] = done.result(1)
        if result:
            raise result

        return Devtools(client, tab, watcher, targets)

    def __getattribute__(self, name: str) -> Any:
        """
        Access a property, raising a ValueError if the instance was previously marked as exited.
        """
        if name != "exited" and object.__getattribute__(self, "exited"):
            raise ValueError("Devtools instance must not be used after __exit__()")
        return object.__getattribute__(self, name)

    def __enter__(self) -> Devtools:
        """
        Enter the `with` context for this instance, raising a ValueError if it was previously marked as exited.
        """
        if self.exited:
            raise ValueError("Devtools instance must not be used after __exit__()")
        return self

    def __exit__(self, exc_type, exc_value, traceback) -> None:
        """
        Exit the `with` context for this instance, disconnecting the client and marking it as exited.
        Does not raise a ValueError if it was previously marked as exited, so you can nest `with` statements.
        """
        if not self.exited:
            # Ignore any return value; we never want to return True to suppress exceptions
            self.client.__exit__(exc_type, exc_value, traceback)
        self.exited = True


@dataclass(frozen=True, order=True)
class Source:
    introduction_type: str
    url: str


# Attach to the thread actor and return it.
def attach_thread(devtools):
    thread_actor = devtools.targets[0]["threadActor"]
    devtools.client.send_receive({"to": thread_actor, "type": "attach"})
    return thread_actor


# Wait for a source matching url_pattern and return its actor
def wait_for_source(devtools, url_pattern, timeout=2):
    source_future = Future()

    def on_source(data):
        for [resource_type, sources] in data.get("array", []):
            if resource_type == "source":
                for source in sources:
                    if url_pattern in source.get("url", ""):
                        source_future.set_result(source["actor"])

    devtools.client.add_event_listener(
        devtools.targets[0]["actor"],
        Events.Watcher.RESOURCES_AVAILABLE_ARRAY,
        on_source,
    )
    devtools.watcher.watch_resources([Resources.SOURCE])
    return source_future.result(timeout)


def set_breakpoint(devtools, source_url, line, column):
    breakpoint_list = devtools.watcher.get_breakpoint_list_actor()
    devtools.client.send_receive(
        {
            "to": breakpoint_list["breakpointList"]["actor"],
            "type": "setBreakpoint",
            "location": {"sourceUrl": source_url, "line": line, "column": column},
        }
    )


# Call a trigger and wait for the debugger to pause and return the data of that paused location
def wait_for_pause(client, thread_actor, trigger, timeout=3):
    future = Future()

    def on_paused(data):
        future.set_result(data)

    client.add_event_listener(thread_actor, "paused", on_paused)
    trigger()
    return future.result(timeout)


# Execute a step and wait for pause.
def step(client, thread_actor, step_type, timeout=3):
    future = Future()

    def on_paused(data):
        future.set_result(data)

    client.add_event_listener(thread_actor, "paused", on_paused)
    client.send_receive({"to": thread_actor, "type": "resume", "resumeLimit": {"type": step_type}})
    return future.result(timeout)


# Resume execution and wait for next pause (e.g: breakpoint)
def resume_and_wait(client, thread_actor, timeout=3):
    future = Future()

    def on_paused(data):
        future.set_result(data)

    client.add_event_listener(thread_actor, "paused", on_paused)
    client.send_receive({"to": thread_actor, "type": "resume"})
    return future.result(timeout)


# Wait for a given number of seconds and assert that no pause happens after calling the trigger
def wait_and_assert_no_pause(client, thread_actor, trigger, duration):
    future = Future()

    def on_paused(data):
        future.set_result(data)

    client.add_event_listener(thread_actor, "paused", on_paused)
    trigger()

    try:
        data = future.result(duration)
        raise AssertionError(f"Received unexpected pause: {data}")
    except TimeoutError:
        pass


# TODO: Use new syntax in python 3.12
# <https://docs.python.org/3/reference/compound_stmts.html#generic-functions>
# <https://docs.python.org/3/library/typing.html#user-defined-generic-types>
T = TypeVar("T")
FrozenMultiset = tuple[tuple[T, int], ...]


def frozen_multiset(items: Iterable[T] = []) -> FrozenMultiset[T]:
    """
    Simulate a frozen multiset using a tuple of tuples.
    Python does not have one yet:
    <https://bugs.python.org/issue40411>
    <https://peps.python.org/pep-0603/>
    """
    # First make a mutable multiset
    result = Counter(items)
    # then convert it to a tuple with a stable order
    return tuple(sorted(result.items()))

def assert_event_listeners(self, node: dict, expected_listeners: Optional[Any], devtools: Devtools):
    if expected_listeners is None:
        self.assertFalse(node["hasEventListeners"])
        return
    self.assertTrue(node["hasEventListeners"])
    nodeActor = NodeActor(devtools.client, node["actor"])
    event_listener_info = nodeActor.get_event_listener_info()
    self.assertEqual(len(event_listener_info), len(expected_listeners))
    for expected_listener, actual_listener in zip(expected_listeners, event_listener_info):
        for key, value in expected_listener.items():
            self.assertEqual(actual_listener[key], value)

def assert_sources_list(
    expected_sources_by_target: Counter[FrozenMultiset[Source]], *, devtools: Optional[Devtools] = None
):
    expected_targets = len(expected_sources_by_target)
    if devtools is None:
        devtools = Devtools.connect(expected_targets=expected_targets)
    with devtools:
        done = Future()
        actual_sources_by_target: Counter[FrozenMultiset[Source]] = Counter()

        def on_source_resource(data):
            for [resource_type, sources] in data["array"]:
                try:
                    assert resource_type == "source"
                    source_urls = frozen_multiset(
                        [Source(source["introductionType"], source["url"]) for source in sources]
                    )
                    assert source_urls not in actual_sources_by_target  # See NOTE above
                    actual_sources_by_target.update([source_urls])
                    if len(actual_sources_by_target) == expected_targets:
                        done.set_result(None)
                except Exception as e:
                    # Raising here does nothing, for some reason.
                    # Send the exception back so it can be raised.
                    done.set_result(e)

        for target in devtools.targets:
            devtools.client.add_event_listener(
                target["actor"],
                Events.Watcher.RESOURCES_AVAILABLE_ARRAY,
                on_source_resource,
            )
        devtools.watcher.watch_resources([Resources.SOURCE])

        result: Optional[Exception] = done.result(1)
        if result:
            raise result
        assert actual_sources_by_target == expected_sources_by_target


def assert_source_content(expected_source: Source, expected_content: str, *, devtools: Optional[Devtools] = None):
    if devtools is None:
        devtools = Devtools.connect()
    with devtools:
        done = Future()
        source_actors = {}

        def on_source_resource(data):
            for [resource_type, sources] in data["array"]:
                try:
                    assert resource_type == "source"
                    for source in sources:
                        if Source(source["introductionType"], source["url"]) == expected_source:
                            source_actors[expected_source] = source["actor"]
                            done.set_result(None)
                except Exception as e:
                    done.set_result(e)

        for target in devtools.targets:
            devtools.client.add_event_listener(
                target["actor"],
                Events.Watcher.RESOURCES_AVAILABLE_ARRAY,
                on_source_resource,
            )
        devtools.watcher.watch_resources([Resources.SOURCE])

        result: Optional[Exception] = done.result(1)
        if result:
            raise result

        # We found at least one source with the given url.
        assert expected_source in source_actors
        source_actor = source_actors[expected_source]

        response = devtools.client.send_receive({"to": source_actor, "type": "source"})

        assert response["source"] == expected_content


def assert_source_breakable_lines_and_positions(
    expected_source: Source,
    expected_breakable_lines: list[int],
    expected_positions: dict[int, list[int]],
    *,
    devtools: Optional[Devtools] = None,
):
    if devtools is None:
        devtools = Devtools.connect()
    with devtools:
        done = Future()
        source_actors = {}

        def on_source_resource(data):
            for [resource_type, sources] in data["array"]:
                try:
                    assert resource_type == "source"
                    for source in sources:
                        if Source(source["introductionType"], source["url"]) == expected_source:
                            source_actors[expected_source] = source["actor"]
                            done.set_result(None)
                except Exception as e:
                    done.set_result(e)

        for target in devtools.targets:
            devtools.client.add_event_listener(
                target["actor"],
                Events.Watcher.RESOURCES_AVAILABLE_ARRAY,
                on_source_resource,
            )
        devtools.watcher.watch_resources([Resources.SOURCE])

        result: Optional[Exception] = done.result(1)
        if result:
            raise result

        # We found at least one source with the given url.
        assert expected_source in source_actors
        source_actor = source_actors[expected_source]

        response = devtools.client.send_receive({"to": source_actor, "type": "getBreakableLines"})
        assert response["lines"] == expected_breakable_lines

        response = devtools.client.send_receive({"to": source_actor, "type": "getBreakpointPositionsCompressed"})
        assert response["positions"] == expected_positions
