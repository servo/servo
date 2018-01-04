A session-fixture which can look at all collected tests
----------------------------------------------------------------

A session-scoped fixture effectively has access to all
collected test items.  Here is an example of a fixture
function which walks all collected tests and looks
if their test class defines a ``callme`` method and
calls it::

    # content of conftest.py

    import pytest

    @pytest.fixture(scope="session", autouse=True)
    def callattr_ahead_of_alltests(request):
        print ("callattr_ahead_of_alltests called")
        seen = set([None])
        session = request.node
        for item in session.items:
            cls = item.getparent(pytest.Class)
            if cls not in seen:
                if hasattr(cls.obj, "callme"):
                   cls.obj.callme()
                seen.add(cls)

test classes may now define a ``callme`` method which
will be called ahead of running any tests::

    # content of test_module.py

    class TestHello(object):
        @classmethod
        def callme(cls):
            print ("callme called!")

        def test_method1(self):
            print ("test_method1 called")

        def test_method2(self):
            print ("test_method1 called")

    class TestOther(object):
        @classmethod
        def callme(cls):
            print ("callme other called")
        def test_other(self):
            print ("test other")

    # works with unittest as well ...
    import unittest

    class SomeTest(unittest.TestCase):
        @classmethod
        def callme(self):
            print ("SomeTest callme called")

        def test_unit1(self):
            print ("test_unit1 method called")

If you run this without output capturing::

    $ pytest -q -s test_module.py
    callattr_ahead_of_alltests called
    callme called!
    callme other called
    SomeTest callme called
    test_method1 called
    .test_method1 called
    .test other
    .test_unit1 method called
    .                                                                 [100%]
    4 passed in 0.12 seconds
