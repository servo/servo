# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from geckordp.actors.root import RootActor

from .utils import Devtools


class TestConnection:
    def test_selected_tab(self, run_servoshell):
        run_servoshell(url="data:text/html,<h1>Test</h1>")

        with Devtools.connect() as devtools:
            root = RootActor(devtools.client)
            tabs = root.list_tabs()

            assert len(tabs) >= 1

            selected_tabs = [tab for tab in tabs if tab.get("selected")]
            assert len(selected_tabs) == 1
