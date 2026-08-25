from .utils import simple_class


class TestSimpleClass:
    """
    Tests for the testing helper function `make_class`.
    """

    def test_returns_class(self):
        """
        Returns a class object.
        """
        assert type is simple_class().__class__

    def test_returns_distinct_classes(self):
        """
        Each call returns a completely new class.
        """
        assert simple_class() is not simple_class()
