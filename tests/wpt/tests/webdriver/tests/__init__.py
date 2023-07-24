import pytest

# Enable pytest assert introspection for assertion helper
pytest.register_assert_rewrite('tests.support.asserts')
