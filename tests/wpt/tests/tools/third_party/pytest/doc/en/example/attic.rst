
.. _`accept example`:

example: specifying and selecting acceptance tests
--------------------------------------------------------------

.. sourcecode:: python

    # ./conftest.py
    def pytest_option(parser):
        group = parser.getgroup("myproject")
        group.addoption(
            "-A", dest="acceptance", action="store_true", help="run (slow) acceptance tests"
        )


    def pytest_funcarg__accept(request):
        return AcceptFixture(request)


    class AcceptFixture:
        def __init__(self, request):
            if not request.config.getoption("acceptance"):
                pytest.skip("specify -A to run acceptance tests")
            self.tmpdir = request.config.mktemp(request.function.__name__, numbered=True)

        def run(self, *cmd):
            """called by test code to execute an acceptance test."""
            self.tmpdir.chdir()
            return subprocess.check_output(cmd).decode()


and the actual test function example:

.. sourcecode:: python

    def test_some_acceptance_aspect(accept):
        accept.tmpdir.mkdir("somesub")
        result = accept.run("ls", "-la")
        assert "somesub" in result

If you run this test without specifying a command line option
the test will get skipped with an appropriate message. Otherwise
you can start to add convenience and test support methods
to your AcceptFixture and drive running of tools or
applications and provide ways to do assertions about
the output.

.. _`decorate a funcarg`:

example: decorating a funcarg in a test module
--------------------------------------------------------------

For larger scale setups it's sometimes useful to decorate
a funcarg just for a particular test module.  We can
extend the `accept example`_ by putting this in our test module:

.. sourcecode:: python

    def pytest_funcarg__accept(request):
        # call the next factory (living in our conftest.py)
        arg = request.getfuncargvalue("accept")
        # create a special layout in our tempdir
        arg.tmpdir.mkdir("special")
        return arg


    class TestSpecialAcceptance:
        def test_sometest(self, accept):
            assert accept.tmpdir.join("special").check()

Our module level factory will be invoked first and it can
ask its request object to call the next factory and then
decorate its result.  This mechanism allows us to stay
ignorant of how/where the function argument is provided -
in our example from a `conftest plugin`_.

sidenote: the temporary directory used here are instances of
the `py.path.local`_ class which provides many of the os.path
methods in a convenient way.

.. _`py.path.local`: ../path.html#local
.. _`conftest plugin`: customize.html#conftestplugin
