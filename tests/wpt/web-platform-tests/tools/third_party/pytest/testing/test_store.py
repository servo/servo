import pytest
from _pytest.store import Store
from _pytest.store import StoreKey


def test_store() -> None:
    store = Store()

    key1 = StoreKey[str]()
    key2 = StoreKey[int]()

    # Basic functionality - single key.
    assert key1 not in store
    store[key1] = "hello"
    assert key1 in store
    assert store[key1] == "hello"
    assert store.get(key1, None) == "hello"
    store[key1] = "world"
    assert store[key1] == "world"
    # Has correct type (no mypy error).
    store[key1] + "string"

    # No interaction with another key.
    assert key2 not in store
    assert store.get(key2, None) is None
    with pytest.raises(KeyError):
        store[key2]
    with pytest.raises(KeyError):
        del store[key2]
    store[key2] = 1
    assert store[key2] == 1
    # Has correct type (no mypy error).
    store[key2] + 20
    del store[key1]
    with pytest.raises(KeyError):
        del store[key1]
    with pytest.raises(KeyError):
        store[key1]

    # setdefault
    store[key1] = "existing"
    assert store.setdefault(key1, "default") == "existing"
    assert store[key1] == "existing"
    key_setdefault = StoreKey[bytes]()
    assert store.setdefault(key_setdefault, b"default") == b"default"
    assert store[key_setdefault] == b"default"

    # Can't accidentally add attributes to store object itself.
    with pytest.raises(AttributeError):
        store.foo = "nope"  # type: ignore[attr-defined]

    # No interaction with anoter store.
    store2 = Store()
    key3 = StoreKey[int]()
    assert key2 not in store2
    store2[key2] = 100
    store2[key3] = 200
    assert store2[key2] + store2[key3] == 300
    assert store[key2] == 1
    assert key3 not in store
