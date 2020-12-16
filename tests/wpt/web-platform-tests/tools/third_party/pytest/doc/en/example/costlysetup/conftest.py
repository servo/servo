# -*- coding: utf-8 -*-
import pytest


@pytest.fixture("session")
def setup(request):
    setup = CostlySetup()
    yield setup
    setup.finalize()


class CostlySetup(object):
    def __init__(self):
        import time

        print("performing costly setup")
        time.sleep(5)
        self.timecostly = 1

    def finalize(self):
        del self.timecostly
