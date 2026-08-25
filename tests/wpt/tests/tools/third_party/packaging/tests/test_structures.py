# This file is dual licensed under the terms of the Apache License, Version
# 2.0, and the BSD License. See the LICENSE file in the root of this repository
# for complete details.

import pytest

from packaging._structures import Infinity, NegativeInfinity


def test_infinity_repr():
    repr(Infinity) == "Infinity"


def test_negative_infinity_repr():
    repr(NegativeInfinity) == "-Infinity"


def test_infinity_hash():
    assert hash(Infinity) == hash(Infinity)


def test_negative_infinity_hash():
    assert hash(NegativeInfinity) == hash(NegativeInfinity)


@pytest.mark.parametrize("left", [1, "a", ("b", 4)])
def test_infinity_comparison(left):
    assert left < Infinity
    assert left <= Infinity
    assert not left == Infinity
    assert left != Infinity
    assert not left > Infinity
    assert not left >= Infinity


@pytest.mark.parametrize("left", [1, "a", ("b", 4)])
def test_negative_infinity_lesser(left):
    assert not left < NegativeInfinity
    assert not left <= NegativeInfinity
    assert not left == NegativeInfinity
    assert left != NegativeInfinity
    assert left > NegativeInfinity
    assert left >= NegativeInfinity


def test_infinity_equal():
    assert Infinity == Infinity


def test_negative_infinity_equal():
    assert NegativeInfinity == NegativeInfinity


def test_negate_infinity():
    assert isinstance(-Infinity, NegativeInfinity.__class__)


def test_negate_negative_infinity():
    assert isinstance(-NegativeInfinity, Infinity.__class__)
