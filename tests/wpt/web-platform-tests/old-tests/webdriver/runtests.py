import unittest

from unittest import TestLoader, TextTestRunner, TestSuite

if __name__ == "__main__":

    loader = TestLoader()
    suite = TestSuite((
        loader.discover(".", pattern="*.py")
        ))

    runner = TextTestRunner(verbosity=2)
    runner.run(suite)
    unittest.main()
