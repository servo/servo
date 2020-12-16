# -*- coding: utf-8 -*-
def test_510(testdir):
    testdir.copy_example("issue_519.py")
    testdir.runpytest("issue_519.py")
