import pytest


@pytest.fixture
def arg2(request):
    pytest.raises(Exception, request.getfixturevalue, "arg1")
