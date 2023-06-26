from pytest_bdd import given
from pytest_bdd import scenario
from pytest_bdd import then
from pytest_bdd import when

import pytest


@scenario("bdd_wallet.feature", "Buy fruits")
def test_publish():
    pass


@pytest.fixture
def wallet():
    class Wallet:
        amount = 0

    return Wallet()


@given("A wallet with 50")
def fill_wallet(wallet):
    wallet.amount = 50


@when("I buy some apples for 1")
def buy_apples(wallet):
    wallet.amount -= 1


@when("I buy some bananas for 2")
def buy_bananas(wallet):
    wallet.amount -= 2


@then("I have 47 left")
def check(wallet):
    assert wallet.amount == 47
