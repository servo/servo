from collections import ChainMap
from collections import Counter
from collections import defaultdict
from collections import deque
from collections import OrderedDict
from dataclasses import dataclass
import textwrap
from types import MappingProxyType
from types import SimpleNamespace
from typing import Any

from _pytest._io.pprint import PrettyPrinter
import pytest


@dataclass
class EmptyDataclass:
    pass


@dataclass
class DataclassWithOneItem:
    foo: str


@dataclass
class DataclassWithTwoItems:
    foo: str
    bar: str


@pytest.mark.parametrize(
    ("data", "expected"),
    (
        pytest.param(
            EmptyDataclass(),
            "EmptyDataclass()",
            id="dataclass-empty",
        ),
        pytest.param(
            DataclassWithOneItem(foo="bar"),
            """
            DataclassWithOneItem(
                foo='bar',
            )
            """,
            id="dataclass-one-item",
        ),
        pytest.param(
            DataclassWithTwoItems(foo="foo", bar="bar"),
            """
            DataclassWithTwoItems(
                foo='foo',
                bar='bar',
            )
            """,
            id="dataclass-two-items",
        ),
        pytest.param(
            {},
            "{}",
            id="dict-empty",
        ),
        pytest.param(
            {"one": 1},
            """
            {
                'one': 1,
            }
            """,
            id="dict-one-item",
        ),
        pytest.param(
            {"one": 1, "two": 2},
            """
            {
                'one': 1,
                'two': 2,
            }
            """,
            id="dict-two-items",
        ),
        pytest.param(OrderedDict(), "OrderedDict()", id="ordereddict-empty"),
        pytest.param(
            OrderedDict({"one": 1}),
            """
            OrderedDict({
                'one': 1,
            })
            """,
            id="ordereddict-one-item",
        ),
        pytest.param(
            OrderedDict({"one": 1, "two": 2}),
            """
            OrderedDict({
                'one': 1,
                'two': 2,
            })
            """,
            id="ordereddict-two-items",
        ),
        pytest.param(
            [],
            "[]",
            id="list-empty",
        ),
        pytest.param(
            [1],
            """
            [
                1,
            ]
            """,
            id="list-one-item",
        ),
        pytest.param(
            [1, 2],
            """
            [
                1,
                2,
            ]
            """,
            id="list-two-items",
        ),
        pytest.param(
            tuple(),
            "()",
            id="tuple-empty",
        ),
        pytest.param(
            (1,),
            """
            (
                1,
            )
            """,
            id="tuple-one-item",
        ),
        pytest.param(
            (1, 2),
            """
            (
                1,
                2,
            )
            """,
            id="tuple-two-items",
        ),
        pytest.param(
            set(),
            "set()",
            id="set-empty",
        ),
        pytest.param(
            {1},
            """
            {
                1,
            }
            """,
            id="set-one-item",
        ),
        pytest.param(
            {1, 2},
            """
            {
                1,
                2,
            }
            """,
            id="set-two-items",
        ),
        pytest.param(
            MappingProxyType({}),
            "mappingproxy({})",
            id="mappingproxy-empty",
        ),
        pytest.param(
            MappingProxyType({"one": 1}),
            """
            mappingproxy({
                'one': 1,
            })
            """,
            id="mappingproxy-one-item",
        ),
        pytest.param(
            MappingProxyType({"one": 1, "two": 2}),
            """
            mappingproxy({
                'one': 1,
                'two': 2,
            })
            """,
            id="mappingproxy-two-items",
        ),
        pytest.param(
            SimpleNamespace(),
            "namespace()",
            id="simplenamespace-empty",
        ),
        pytest.param(
            SimpleNamespace(one=1),
            """
            namespace(
                one=1,
            )
            """,
            id="simplenamespace-one-item",
        ),
        pytest.param(
            SimpleNamespace(one=1, two=2),
            """
            namespace(
                one=1,
                two=2,
            )
            """,
            id="simplenamespace-two-items",
        ),
        pytest.param(
            defaultdict(str), "defaultdict(<class 'str'>, {})", id="defaultdict-empty"
        ),
        pytest.param(
            defaultdict(str, {"one": "1"}),
            """
            defaultdict(<class 'str'>, {
                'one': '1',
            })
            """,
            id="defaultdict-one-item",
        ),
        pytest.param(
            defaultdict(str, {"one": "1", "two": "2"}),
            """
            defaultdict(<class 'str'>, {
                'one': '1',
                'two': '2',
            })
            """,
            id="defaultdict-two-items",
        ),
        pytest.param(
            Counter(),
            "Counter()",
            id="counter-empty",
        ),
        pytest.param(
            Counter("1"),
            """
            Counter({
                '1': 1,
            })
            """,
            id="counter-one-item",
        ),
        pytest.param(
            Counter("121"),
            """
            Counter({
                '1': 2,
                '2': 1,
            })
            """,
            id="counter-two-items",
        ),
        pytest.param(ChainMap(), "ChainMap({})", id="chainmap-empty"),
        pytest.param(
            ChainMap({"one": 1, "two": 2}),
            """
            ChainMap(
                {
                    'one': 1,
                    'two': 2,
                },
            )
            """,
            id="chainmap-one-item",
        ),
        pytest.param(
            ChainMap({"one": 1}, {"two": 2}),
            """
            ChainMap(
                {
                    'one': 1,
                },
                {
                    'two': 2,
                },
            )
            """,
            id="chainmap-two-items",
        ),
        pytest.param(
            deque(),
            "deque([])",
            id="deque-empty",
        ),
        pytest.param(
            deque([1]),
            """
            deque([
                1,
            ])
            """,
            id="deque-one-item",
        ),
        pytest.param(
            deque([1, 2]),
            """
            deque([
                1,
                2,
            ])
            """,
            id="deque-two-items",
        ),
        pytest.param(
            deque([1, 2], maxlen=3),
            """
            deque(maxlen=3, [
                1,
                2,
            ])
            """,
            id="deque-maxlen",
        ),
        pytest.param(
            {
                "chainmap": ChainMap({"one": 1}, {"two": 2}),
                "counter": Counter("122"),
                "dataclass": DataclassWithTwoItems(foo="foo", bar="bar"),
                "defaultdict": defaultdict(str, {"one": "1", "two": "2"}),
                "deque": deque([1, 2], maxlen=3),
                "dict": {"one": 1, "two": 2},
                "list": [1, 2],
                "mappingproxy": MappingProxyType({"one": 1, "two": 2}),
                "ordereddict": OrderedDict({"one": 1, "two": 2}),
                "set": {1, 2},
                "simplenamespace": SimpleNamespace(one=1, two=2),
                "tuple": (1, 2),
            },
            """
            {
                'chainmap': ChainMap(
                    {
                        'one': 1,
                    },
                    {
                        'two': 2,
                    },
                ),
                'counter': Counter({
                    '2': 2,
                    '1': 1,
                }),
                'dataclass': DataclassWithTwoItems(
                    foo='foo',
                    bar='bar',
                ),
                'defaultdict': defaultdict(<class 'str'>, {
                    'one': '1',
                    'two': '2',
                }),
                'deque': deque(maxlen=3, [
                    1,
                    2,
                ]),
                'dict': {
                    'one': 1,
                    'two': 2,
                },
                'list': [
                    1,
                    2,
                ],
                'mappingproxy': mappingproxy({
                    'one': 1,
                    'two': 2,
                }),
                'ordereddict': OrderedDict({
                    'one': 1,
                    'two': 2,
                }),
                'set': {
                    1,
                    2,
                },
                'simplenamespace': namespace(
                    one=1,
                    two=2,
                ),
                'tuple': (
                    1,
                    2,
                ),
            }
            """,
            id="deep-example",
        ),
    ),
)
def test_consistent_pretty_printer(data: Any, expected: str) -> None:
    assert PrettyPrinter().pformat(data) == textwrap.dedent(expected).strip()
