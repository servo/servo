from _pytest.stash import Stash
from _pytest.stash import StashKey
import pytest


def test_stash() -> None:
    stash = Stash()

    assert len(stash) == 0
    assert not stash

    key1 = StashKey[str]()
    key2 = StashKey[int]()

    # Basic functionality - single key.
    assert key1 not in stash
    stash[key1] = "hello"
    assert key1 in stash
    assert stash[key1] == "hello"
    assert stash.get(key1, None) == "hello"
    stash[key1] = "world"
    assert stash[key1] == "world"
    # Has correct type (no mypy error).
    stash[key1] + "string"
    assert len(stash) == 1
    assert stash

    # No interaction with another key.
    assert key2 not in stash
    assert stash.get(key2, None) is None
    with pytest.raises(KeyError):
        stash[key2]
    with pytest.raises(KeyError):
        del stash[key2]
    stash[key2] = 1
    assert stash[key2] == 1
    # Has correct type (no mypy error).
    stash[key2] + 20
    del stash[key1]
    with pytest.raises(KeyError):
        del stash[key1]
    with pytest.raises(KeyError):
        stash[key1]

    # setdefault
    stash[key1] = "existing"
    assert stash.setdefault(key1, "default") == "existing"
    assert stash[key1] == "existing"
    key_setdefault = StashKey[bytes]()
    assert stash.setdefault(key_setdefault, b"default") == b"default"
    assert stash[key_setdefault] == b"default"
    assert len(stash) == 3
    assert stash

    # Can't accidentally add attributes to stash object itself.
    with pytest.raises(AttributeError):
        stash.foo = "nope"  # type: ignore[attr-defined]

    # No interaction with another stash.
    stash2 = Stash()
    key3 = StashKey[int]()
    assert key2 not in stash2
    stash2[key2] = 100
    stash2[key3] = 200
    assert stash2[key2] + stash2[key3] == 300
    assert stash[key2] == 1
    assert key3 not in stash
