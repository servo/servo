from unittest import TestCase  # noqa: F401


for i in range(15000):
    exec(
        f"""
class Test{i}(TestCase):
    @classmethod
    def setUpClass(cls): pass
    def test_1(self): pass
    def test_2(self): pass
    def test_3(self): pass
"""
    )
