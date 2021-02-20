import unittest

import websockets


combined_exports = (
    websockets.auth.__all__
    + websockets.client.__all__
    + websockets.exceptions.__all__
    + websockets.protocol.__all__
    + websockets.server.__all__
    + websockets.typing.__all__
    + websockets.uri.__all__
)


class TestExportsAllSubmodules(unittest.TestCase):
    def test_top_level_module_reexports_all_submodule_exports(self):
        self.assertEqual(set(combined_exports), set(websockets.__all__))

    def test_submodule_exports_are_globally_unique(self):
        self.assertEqual(len(set(combined_exports)), len(combined_exports))
