# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from concurrent.futures import Future
from geckordp.actors.root import RootActor
from geckordp.actors.descriptors.tab import TabActor
from geckordp.actors.watcher import WatcherActor
from geckordp.actors.resources import Resources
from geckordp.actors.events import Events
from geckordp.rdp_client import RDPClient
import http.server
import os.path
import socketserver
import subprocess
import sys
import time
from threading import Thread
from typing import Optional


def run_tests(script_path):
    run_test(sources_test, os.path.join(script_path, "devtools_tests/sources"))


def run_test(test_fun, test_dir):
    print(f">>> {test_dir}", file=sys.stderr)
    server = None
    base_url = Future()

    class Handler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=test_dir, **kwargs)

        def log_message(self, format, *args):
            # Uncomment this to log requests.
            # return super().log_message(format, *args)
            pass

    def server_thread():
        nonlocal server
        server = socketserver.TCPServer(("0.0.0.0", 0), Handler)
        base_url.set_result(f"http://127.0.0.1:{server.server_address[1]}")
        server.serve_forever()

    # Start a web server for the test.
    thread = Thread(target=server_thread)
    thread.start()
    base_url = base_url.result(1)

    # Change this setting if you want to debug Servo.
    os.environ["RUST_LOG"] = "error,devtools=warn"

    # Run servoshell.
    servoshell = subprocess.Popen(["target/release/servo", "--devtools=6080", f"{base_url}/test.html"])

    # FIXME: Don’t do this
    time.sleep(1)

    try:
        client = RDPClient()
        client.connect("127.0.0.1", 6080)
        test_fun(client, base_url)
    except Exception as e:
        raise e
    finally:
        # Terminate servoshell.
        servoshell.terminate()

        # Stop the web server.
        server.shutdown()
        thread.join()


def sources_test(client, base_url):
    root = RootActor(client)
    tabs = root.list_tabs()
    tab_dict = tabs[0]
    tab = TabActor(client, tab_dict["actor"])
    watcher = tab.get_watcher()
    watcher = WatcherActor(client, watcher["actor"])

    target = Future()

    def on_target(data):
        if data["target"]["browsingContextID"] == tab_dict["browsingContextID"]:
            target.set_result(data["target"])

    client.add_event_listener(
        watcher.actor_id, Events.Watcher.TARGET_AVAILABLE_FORM, on_target,
    )
    watcher.watch_targets(WatcherActor.Targets.FRAME)

    done = Future()
    target = target.result(1)

    def on_source_resource(data):
        for [resource_type, sources] in data["array"]:
            try:
                assert resource_type == "source"
                assert [source["url"] for source in sources] == [f"{base_url}/classic.js", f"{base_url}/test.html", "https://servo.org/js/load-table.js"]
                done.set_result(None)
            except Exception as e:
                # Raising here does nothing, for some reason.
                # Send the exception back so it can be raised.
                done.set_result(e)

    client.add_event_listener(
        target["actor"],
        Events.Watcher.RESOURCES_AVAILABLE_ARRAY,
        on_source_resource,
    )
    watcher.watch_resources([Resources.SOURCE])

    result: Optional[Exception] = done.result(1)
    if result:
        raise result
    client.disconnect()
