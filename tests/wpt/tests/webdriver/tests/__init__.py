import pytest

# Enable pytest assert introspection for assertion helper
pytest.register_assert_rewrite('tests.bidi')
pytest.register_assert_rewrite('tests.support.asserts')
pytest.register_assert_rewrite('tests.support.classic.asserts')
