# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from concurrent.futures import Future

from geckordp.actors.events import Events
from geckordp.actors.resources import Resources

from .utils import Devtools, DevtoolsTestCase


class NetworkTests(DevtoolsTestCase):
    def test_navigation(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/tab/page1.html")
        with Devtools.connect() as devtools:
            for message_data, target_path in [
                ({"type": "navigateTo", "url": f"{self.base_urls[0]}/tab/page2.html"}, "/tab/page2.html"),
                ({"type": "goBack"}, "/tab/page1.html"),
                ({"type": "goForward"}, "/tab/page2.html"),
            ]:
                done = Future()

                def on_target_available(data):
                    if data.get("target", {}).get("url", "").endswith(target_path):
                        done.set_result(None)

                devtools.client.add_event_listener(
                    devtools.watcher.actor_id,
                    Events.Watcher.TARGET_AVAILABLE_FORM,
                    on_target_available,
                )
                devtools.client.send_receive({"to": devtools.tab.actor_id, **message_data})

                done.result(1)

    def test_stylesheet_inline(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/stylesheets/inline_style.html")
        with Devtools.connect() as devtools:
            done = Future()
            stylesheets_data = []

            def on_resource(data):
                for [resource_type, resources] in data["array"]:
                    if resource_type == "stylesheet":
                        stylesheets_data.extend(resources)
                        done.set_result(None)

            devtools.client.add_event_listener(
                devtools.targets[0]["actor"],
                Events.Watcher.RESOURCES_AVAILABLE_ARRAY,
                on_resource,
            )
            devtools.watcher.watch_resources([Resources.STYLESHEET])
            done.result(1)

            # Inline sheets won't have href.
            inline_sheet = stylesheets_data[0]
            self.assertIsNone(inline_sheet.get("href"))
            self.assertEqual(inline_sheet["ruleCount"], 2)
            self.assertFalse(inline_sheet["system"])
            self.assertFalse(inline_sheet["disabled"])

    def test_stylesheet_linked(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/stylesheets/linked_style.html")
        with Devtools.connect() as devtools:
            done = Future()
            stylesheets_data = []

            def on_resource(data):
                for [resource_type, resources] in data["array"]:
                    if resource_type == "stylesheet":
                        stylesheets_data.extend(resources)
                        done.set_result(None)

            devtools.client.add_event_listener(
                devtools.targets[0]["actor"],
                Events.Watcher.RESOURCES_AVAILABLE_ARRAY,
                on_resource,
            )
            devtools.watcher.watch_resources([Resources.STYLESHEET])
            done.result(1)

            # Linked sheets have linked css as href.
            linked_sheet = stylesheets_data[0]
            self.assertEqual(f"{self.base_urls[0]}/stylesheets/styles.css", linked_sheet["href"])
            self.assertFalse(linked_sheet["system"])
            self.assertEqual(linked_sheet["ruleCount"], 1)
            self.assertFalse(linked_sheet["disabled"])

    def test_stylesheet_content(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/stylesheets/linked_style.html")
        with Devtools.connect() as devtools:
            founded_resources = []
            done = Future()

            def on_resource(data):
                for [resource_type, resources] in data["array"]:
                    if resource_type == "stylesheet":
                        founded_resources.extend(resources)
                        done.set_result(None)

            devtools.client.add_event_listener(
                devtools.targets[0]["actor"],
                Events.Watcher.RESOURCES_AVAILABLE_ARRAY,
                on_resource,
            )
            devtools.watcher.watch_resources([Resources.STYLESHEET])
            done.result(1)

            # Test getText by sending the resource id.
            reply = devtools.client.send_receive(
                {
                    "to": devtools.targets[0]["styleSheetsActor"],
                    "type": "getText",
                    "resourceId": founded_resources[0]["resourceId"],
                }
            )
            style_text = reply["text"]["initial"]
            self.assertIn("body { background: green; font-size: small; }", style_text)
