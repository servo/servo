# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import annotations
from typing import Optional, Any
from concurrent.futures import Future
import time

from geckordp.actors.events import Events
from geckordp.actors.inspector import InspectorActor
from geckordp.actors.node import NodeActor
from geckordp.actors.string import StringActor
from geckordp.actors.walker import WalkerActor
from geckordp.actors.web_console import WebConsoleActor

from .utils import Devtools


class TestInspectorTab:
    def test_inspector_event_listeners(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/inspector/event_listeners.html")
        with Devtools.connect() as devtools:
            inspector = InspectorActor(devtools.client, devtools.targets[0]["inspectorActor"])
            walker = WalkerActor(devtools.client, inspector.get_walker()["actor"])
            document_element = walker.document_element("")["actor"]

            button = walker.query_selector(document_element, "button")["node"]
            span = walker.query_selector(document_element, "span")["node"]
            div = walker.query_selector(document_element, "div")["node"]

            def assert_event_listeners(node: dict, expected_listeners: Optional[Any], devtools: Devtools):
                if expected_listeners is None:
                    assert not node["hasEventListeners"]
                    return

                assert node["hasEventListeners"]
                node_actor = NodeActor(devtools.client, node["actor"])
                event_listener_info = node_actor.get_event_listener_info()
                actual = [{"type": e["type"], "capturing": e["capturing"]} for e in event_listener_info]
                assert actual == expected_listeners

            assert_event_listeners(button, [{"type": "click", "capturing": False}], devtools)
            assert_event_listeners(span, [{"type": "hover", "capturing": True}], devtools)
            assert_event_listeners(div, None, devtools)

    def test_inspector_attribute_modifications_affect_dom(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/inspector/demo_dom.html")
        with Devtools.connect() as devtools:
            inspector = InspectorActor(devtools.client, devtools.targets[0]["inspectorActor"])
            walker = WalkerActor(devtools.client, inspector.get_walker()["actor"])
            document_element = walker.document_element("")["actor"]
            body = walker.query_selector(document_element, "body")["node"]["actor"]

            mutation_result = Future()

            async def on_new_mutations(data):
                mutation_result.set_result(data)

            devtools.client.add_event_listener(
                inspector.get_walker()["actor"], Events.Walker.NEW_MUTATIONS, on_new_mutations
            )

            # Assert that the initial state is correct
            first_child = walker.children(body)[0]
            assert first_child["attrs"] == [{"name": "foo", "value": "bar"}]

            # Modify the nodes attribute
            NodeActor(devtools.client, first_child["actor"]).modify_attributes(
                [{"attributeName": "foo", "newValue": "baz"}]
            )

            # Wait for the mutation notification to arrive
            mutation_result.result(1)

            # Assert that the notification is correct
            assert walker.get_mutations(False) == [
                {"attributeName": "foo", "newValue": "baz", "type": "attributes", "target": first_child["actor"]}
            ]

            # Assert that the new DOM state is correct
            assert walker.children(body)[0]["attrs"] == [{"name": "foo", "value": "baz"}]

    def test_inspector_notices_attribute_mutation_from_javascript(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/inspector/demo_dom.html")
        with Devtools.connect() as devtools:
            inspector = InspectorActor(devtools.client, devtools.targets[0]["inspectorActor"])
            walker = WalkerActor(devtools.client, inspector.get_walker()["actor"])
            document_element = walker.document_element("")["actor"]
            console = WebConsoleActor(devtools.client, devtools.targets[0]["consoleActor"])
            body = walker.query_selector(document_element, "body")["node"]["actor"]

            mutation_result = Future()
            evaluation_result = Future()

            async def on_new_mutations(data):
                mutation_result.set_result(data)

            async def on_evaluation_result(data: dict):
                evaluation_result.set_result(data)

            devtools.client.add_event_listener(
                inspector.get_walker()["actor"], Events.Walker.NEW_MUTATIONS, on_new_mutations
            )
            devtools.client.add_event_listener(
                console.actor_id, Events.WebConsole.EVALUATION_RESULT, on_evaluation_result
            )

            # Modify the nodes attribute
            target = walker.children(body)[0]
            console.evaluate_js_async("document.body.firstElementChild.setAttribute('foo', 'baz');")
            evaluation_result.result(1)

            # Wait for the mutation notification to arrive
            mutation_result.result(1)

            # Assert that the notification is correct
            assert walker.get_mutations(False) == [
                {"attributeName": "foo", "newValue": "baz", "type": "attributes", "target": target["actor"]}
            ]

    def test_inspector_doesnt_crash_when_attribute_on_element_it_doesnt_know_about_is_mutated(
        self, run_servoshell, web_server_urls
    ):
        run_servoshell(url=f"{web_server_urls[0]}/inspector/demo_dom.html")
        with Devtools.connect() as devtools:
            inspector = InspectorActor(devtools.client, devtools.targets[0]["inspectorActor"])
            walker = WalkerActor(devtools.client, inspector.get_walker()["actor"])
            console = WebConsoleActor(devtools.client, devtools.targets[0]["consoleActor"])

            did_see_new_mutations = False
            evaluation_result = Future()

            async def on_new_mutations(data):
                nonlocal did_see_new_mutations
                did_see_new_mutations = True

            async def on_evaluation_result(data: dict):
                evaluation_result.set_result(data)

            devtools.client.add_event_listener(
                inspector.get_walker()["actor"], Events.Walker.NEW_MUTATIONS, on_new_mutations
            )
            devtools.client.add_event_listener(
                console.actor_id, Events.WebConsole.EVALUATION_RESULT, on_evaluation_result
            )

            # Modify the nodes attribute
            console.evaluate_js_async("document.body.firstElementChild.setAttribute('foo', 'baz');")
            evaluation_result.result(1)

            # Wait for a bit for unwanted notifications to arrive - we should not get any.
            time.sleep(1)
            assert not did_see_new_mutations
            assert walker.get_mutations(False) == []

    def test_walker_observes_new_dom_after_nav(self, run_servoshell, web_server_urls):
        # This tests that the walker actor can correctly recognize a new DOM across distinct
        # pipelines and script threads. It does not exercise the full exchange of messages required
        # for the Firefox toolbox to successfully refresh its inspector panel.

        run_servoshell(url=f"{web_server_urls[0]}/tab/page1.html")
        with Devtools.connect() as devtools:
            target_destroyed = Future()
            target_available = Future()

            def on_target_destroyed(_):
                target_destroyed.set_result(None)

            def on_target_available(data):
                target = data.get("target", {})
                if target.get("url", "").endswith("/tab/page2.html"):
                    target_available.set_result(target)

            devtools.client.add_event_listener(
                devtools.watcher.actor_id,
                Events.Watcher.TARGET_DESTROYED_FORM,
                on_target_destroyed,
            )
            devtools.client.add_event_listener(
                devtools.watcher.actor_id,
                Events.Watcher.TARGET_AVAILABLE_FORM,
                on_target_available,
            )
            devtools.client.send_receive(
                {
                    "to": devtools.tab.actor_id,
                    "type": "navigateTo",
                    # Use a different base URL to test walker across script threads.
                    "url": f"{web_server_urls[1]}/tab/page2.html",
                },
            )
            target_destroyed.result(1)
            new_target = target_available.result(1)

            inspector = InspectorActor(devtools.client, new_target["inspectorActor"])
            walker_info = inspector.get_walker()
            walker = WalkerActor(devtools.client, walker_info["actor"])
            root_node = walker_info["root"]["actor"]

            title_node = walker.query_selector(root_node, "title")
            assert title_node.get("node") is not None
            assert title_node["node"].get("inlineTextChild") is not None
            assert title_node["node"]["inlineTextChild"].get("nodeValue") == "Page 2"

    def test_watcher_returns_same_breakpoint_list_actor_every_time(self, run_servoshell):
        run_servoshell(url="data:text/html,")
        with Devtools.connect() as devtools:
            response1 = devtools.watcher.get_breakpoint_list_actor()
            response2 = devtools.watcher.get_breakpoint_list_actor()
            assert response1["breakpointList"]["actor"] == response2["breakpointList"]["actor"]

    def test_watcher_returns_same_blackboxing_actor_every_time(self, run_servoshell):
        run_servoshell(url="data:text/html,")
        with Devtools.connect() as devtools:
            response1 = devtools.watcher.get_blackboxing_actor()
            response2 = devtools.watcher.get_blackboxing_actor()
            assert response1["blackboxing"]["actor"] == response2["blackboxing"]["actor"]

    def test_inspector_inner_html(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/inspector/event_listeners.html")
        with Devtools.connect() as devtools:
            inspector = InspectorActor(devtools.client, devtools.targets[0]["inspectorActor"])
            walker_info = inspector.get_walker()
            walker = WalkerActor(devtools.client, inspector.get_walker()["actor"])

            root_node = walker_info["root"]["actor"]
            target_node = walker.query_selector(root_node, "button")["node"]["actor"]

            inner_html = walker.inner_html(target_node)
            assert inner_html["initial"] == "Button"

            target_node = walker.query_selector(root_node, "div")["node"]["actor"]
            inner_html = walker.inner_html(target_node)
            string_actor = StringActor(devtools.client, inner_html["actor"])
            assert len(string_actor.substring(0, inner_html["length"])) == inner_html["length"]

    def test_inspector_outer_html(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/inspector/event_listeners.html")
        with Devtools.connect() as devtools:
            inspector = InspectorActor(devtools.client, devtools.targets[0]["inspectorActor"])
            walker_info = inspector.get_walker()
            walker = WalkerActor(devtools.client, inspector.get_walker()["actor"])

            root_node = walker_info["root"]["actor"]
            target_node = walker.query_selector(root_node, "button")["node"]["actor"]

            inner_html = walker.outer_html(target_node)
            assert inner_html["initial"] == "<button>Button</button>"
