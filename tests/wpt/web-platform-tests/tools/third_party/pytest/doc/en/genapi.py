import textwrap
import inspect

class Writer(object):
    def __init__(self, clsname):
        self.clsname = clsname

    def __enter__(self):
        self.file = open("%s.api" % self.clsname, "w")
        return self

    def __exit__(self, *args):
        self.file.close()
        print "wrote", self.file.name

    def line(self, line):
        self.file.write(line+"\n")

    def docmethod(self, method):
        doc = " ".join(method.__doc__.split())
        indent = "         "
        w = textwrap.TextWrapper(initial_indent=indent,
                                 subsequent_indent=indent)

        spec = inspect.getargspec(method)
        del spec.args[0]
        self.line(".. py:method:: " + method.__name__ +
                  inspect.formatargspec(*spec))
        self.line("")
        self.line(w.fill(doc))
        self.line("")

def pytest_funcarg__a(request):
    with Writer("request") as writer:
        writer.docmethod(request.getfixturevalue)
        writer.docmethod(request.cached_setup)
        writer.docmethod(request.addfinalizer)
        writer.docmethod(request.applymarker)

def test_hello(a):
    pass
