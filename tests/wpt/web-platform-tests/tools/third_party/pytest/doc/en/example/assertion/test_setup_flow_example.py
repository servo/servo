def setup_module(module):
    module.TestStateFullThing.classcount = 0


class TestStateFullThing(object):

    def setup_class(cls):
        cls.classcount += 1

    def teardown_class(cls):
        cls.classcount -= 1

    def setup_method(self, method):
        self.id = eval(method.__name__[5:])

    def test_42(self):
        assert self.classcount == 1
        assert self.id == 42

    def test_23(self):
        assert self.classcount == 1
        assert self.id == 23


def teardown_module(module):
    assert module.TestStateFullThing.classcount == 0


""" For this example the control flow happens as follows::
    import test_setup_flow_example
    setup_module(test_setup_flow_example)
       setup_class(TestStateFullThing)
           instance = TestStateFullThing()
           setup_method(instance, instance.test_42)
              instance.test_42()
           setup_method(instance, instance.test_23)
              instance.test_23()
       teardown_class(TestStateFullThing)
    teardown_module(test_setup_flow_example)

Note that ``setup_class(TestStateFullThing)`` is called and not
``TestStateFullThing.setup_class()`` which would require you
to insert ``setup_class = classmethod(setup_class)`` to make
your setup function callable.
"""
