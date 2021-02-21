import pytest
import yaml

from tools.ci.tc import taskgraph

@pytest.mark.parametrize("data, update_data, expected", [
    ({"a": 1}, {"b": 2}, {"a": 1, "b": 2}),
    ({"a": 1}, {"a": 2}, {"a": 2}),
    ({"a": [1]}, {"a": [2]}, {"a": [1, 2]}),
    ({"a": {"b": 1, "c": 2}}, {"a": {"b": 2, "d": 3}}, {"a": {"b": 2, "c": 2, "d": 3}}),
    ({"a": {"b": [1]}}, {"a": {"b": [2]}}, {"a": {"b": [1, 2]}}),
]
)
def test_update_recursive(data, update_data, expected):
    taskgraph.update_recursive(data, update_data)
    assert data == expected


def test_use():
    data = """
components:
  component1:
    a: 1
    b: [1]
    c: "c"
  component2:
    a: 2
    b: [2]
    d: "d"
tasks:
  - task1:
      use:
       - component1
       - component2
      b: [3]
      c: "e"
"""
    tasks_data = yaml.safe_load(data)
    assert taskgraph.load_tasks(tasks_data) == {
        "task1": {
            "a": 2,
            "b": [1,2,3],
            "c": "e",
            "d": "d",
            "name": "task1"
        }
    }


def test_var():
    data = """
components:
  component1:
    a: ${vars.value}
tasks:
  - task1:
      use:
       - component1
      vars:
        value: 1
"""
    tasks_data = yaml.safe_load(data)
    assert taskgraph.load_tasks(tasks_data) == {
        "task1": {
            "a": "1",
            "vars": {"value": 1},
            "name": "task1"
        }
    }


def test_map():
    data = """
components: {}
tasks:
 - $map:
     for:
       - vars:
           a: 1
         b: [1]
       - vars:
           a: 2
         b: [2]
     do:
       - task1-${vars.a}:
           a: ${vars.a}
           b: [3]
       - task2-${vars.a}:
           a: ${vars.a}
           b: [4]
"""
    tasks_data = yaml.safe_load(data)
    assert taskgraph.load_tasks(tasks_data) == {
        "task1-1": {
            "a": "1",
            "b": [1, 3],
            "vars": {"a": 1},
            "name": "task1-1"
        },
        "task1-2": {
            "a": "2",
            "b": [2, 3],
            "vars": {"a": 2},
            "name": "task1-2"
        },
        "task2-1": {
            "a": "1",
            "b": [1, 4],
            "vars": {"a": 1},
            "name": "task2-1"
        },
        "task2-2": {
            "a": "2",
            "b": [2, 4],
            "vars": {"a": 2},
            "name": "task2-2"
        },

    }


def test_chunks():
    data = """
components: {}
tasks:
  - task1:
      name: task1-${chunks.id}
      chunks: 2
"""
    tasks_data = yaml.safe_load(data)
    assert taskgraph.load_tasks(tasks_data) == {
        "task1-1": {
            "name": "task1-1",
            "chunks": {
                "id": 1,
                "total": 2
            }
        },
        "task1-2": {
            "name": "task1-2",
            "chunks": {
                "id": 2,
                "total": 2
            }
        }
    }
