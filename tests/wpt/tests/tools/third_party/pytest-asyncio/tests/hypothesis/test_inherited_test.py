import hypothesis.strategies as st
import pytest
from hypothesis import given


class BaseClass:
    @pytest.mark.asyncio
    @given(value=st.integers())
    async def test_hypothesis(self, value: int) -> None:
        pass


class TestOne(BaseClass):
    """During the first execution the Hypothesis test
    is wrapped in a synchronous function."""


class TestTwo(BaseClass):
    """Execute the test a second time to ensure that
    the test receives a fresh event loop."""
