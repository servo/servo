# mypy: allow-untyped-defs

from .. import expectedtree, metadata
from collections import defaultdict


def dump_tree(tree):
    rv = []

    def dump_node(node, indent=0):
        prefix = " " * indent
        if not node.prop:
            data = "root"
        else:
            data = f"{node.prop}:{node.value}"
        if node.result_values:
            data += " result_values:%s" % (",".join(sorted(node.result_values)))
        rv.append(f"{prefix}<{data}>")
        for child in sorted(node.children, key=lambda x:x.value):
            dump_node(child, indent + 2)
    dump_node(tree)
    return "\n".join(rv)


def results_object(results):
    results_obj = defaultdict(lambda: defaultdict(int))
    for run_info, status in results:
        run_info = metadata.RunInfo(run_info)
        results_obj[run_info][status] += 1
    return results_obj


def test_build_tree_0():
    # Pass if debug
    results = [({"os": "linux", "version": "18.04", "debug": True}, "FAIL"),
               ({"os": "linux", "version": "18.04", "debug": False}, "PASS"),
               ({"os": "linux", "version": "16.04", "debug": False}, "PASS"),
               ({"os": "mac", "version": "10.12", "debug": True}, "FAIL"),
               ({"os": "mac", "version": "10.12", "debug": False}, "PASS"),
               ({"os": "win", "version": "7", "debug": False}, "PASS"),
               ({"os": "win", "version": "10", "debug": False}, "PASS")]
    results_obj = results_object(results)
    tree = expectedtree.build_tree(["os", "version", "debug"], {}, results_obj)

    expected = """<root>
  <debug:False result_values:PASS>
  <debug:True result_values:FAIL>"""

    assert dump_tree(tree) == expected


def test_build_tree_1():
    # Pass if linux or windows 10
    results = [({"os": "linux", "version": "18.04", "debug": True}, "PASS"),
               ({"os": "linux", "version": "18.04", "debug": False}, "PASS"),
               ({"os": "linux", "version": "16.04", "debug": False}, "PASS"),
               ({"os": "mac", "version": "10.12", "debug": True}, "FAIL"),
               ({"os": "mac", "version": "10.12", "debug": False}, "FAIL"),
               ({"os": "win", "version": "7", "debug": False}, "FAIL"),
               ({"os": "win", "version": "10", "debug": False}, "PASS")]
    results_obj = results_object(results)
    tree = expectedtree.build_tree(["os", "debug"], {"os": ["version"]}, results_obj)

    expected = """<root>
  <os:linux result_values:PASS>
  <os:mac result_values:FAIL>
  <os:win>
    <version:10 result_values:PASS>
    <version:7 result_values:FAIL>"""

    assert dump_tree(tree) == expected


def test_build_tree_2():
    # Fails in a specific configuration
    results = [({"os": "linux", "version": "18.04", "debug": True}, "PASS"),
               ({"os": "linux", "version": "18.04", "debug": False}, "FAIL"),
               ({"os": "linux", "version": "16.04", "debug": False}, "PASS"),
               ({"os": "linux", "version": "16.04", "debug": True}, "PASS"),
               ({"os": "mac", "version": "10.12", "debug": True}, "PASS"),
               ({"os": "mac", "version": "10.12", "debug": False}, "PASS"),
               ({"os": "win", "version": "7", "debug": False}, "PASS"),
               ({"os": "win", "version": "10", "debug": False}, "PASS")]
    results_obj = results_object(results)
    tree = expectedtree.build_tree(["os", "debug"], {"os": ["version"]}, results_obj)

    expected = """<root>
  <os:linux>
    <debug:False>
      <version:16.04 result_values:PASS>
      <version:18.04 result_values:FAIL>
    <debug:True result_values:PASS>
  <os:mac result_values:PASS>
  <os:win result_values:PASS>"""

    assert dump_tree(tree) == expected


def test_build_tree_3():

    results = [({"os": "linux", "version": "18.04", "debug": True, "unused": False}, "PASS"),
               ({"os": "linux", "version": "18.04", "debug": True, "unused": True}, "FAIL")]
    results_obj = results_object(results)
    tree = expectedtree.build_tree(["os", "debug"], {"os": ["version"]}, results_obj)

    expected = """<root result_values:FAIL,PASS>"""

    assert dump_tree(tree) == expected


def test_build_tree_4():
    # Check counts for multiple statuses
    results = [({"os": "linux", "version": "18.04", "debug": False}, "FAIL"),
               ({"os": "linux", "version": "18.04", "debug": False}, "PASS"),
               ({"os": "linux", "version": "18.04", "debug": False}, "PASS")]
    results_obj = results_object(results)
    tree = expectedtree.build_tree(["os", "version", "debug"], {}, results_obj)

    assert tree.result_values["PASS"] == 2
    assert tree.result_values["FAIL"] == 1
