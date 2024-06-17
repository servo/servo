# SPDX-License-Identifier: MIT


class TestImportStar:
    def test_from_attr_import_star(self):
        """
        import * from attr
        """
        # attr_import_star contains `from attr import *`, which cannot
        # be done here because *-imports are only allowed on module level.
        from . import attr_import_star  # noqa: F401
