
from pluggy import _TagTracer


def test_simple():
    rootlogger = _TagTracer()
    log = rootlogger.get("pytest")
    log("hello")
    out = []
    rootlogger.setwriter(out.append)
    log("world")
    assert len(out) == 1
    assert out[0] == "world [pytest]\n"
    sublog = log.get("collection")
    sublog("hello")
    assert out[1] == "hello [pytest:collection]\n"


def test_indent():
    rootlogger = _TagTracer()
    log = rootlogger.get("1")
    out = []
    log.root.setwriter(lambda arg: out.append(arg))
    log("hello")
    log.root.indent += 1
    log("line1")
    log("line2")
    log.root.indent += 1
    log("line3")
    log("line4")
    log.root.indent -= 1
    log("line5")
    log.root.indent -= 1
    log("last")
    assert len(out) == 7
    names = [x[:x.rfind(' [')] for x in out]
    assert names == [
        'hello', '  line1', '  line2',
        '    line3', '    line4', '  line5', 'last']


def test_readable_output_dictargs():
    rootlogger = _TagTracer()

    out = rootlogger.format_message(['test'], [1])
    assert out == ['1 [test]\n']

    out2 = rootlogger.format_message(['test'], ['test', {'a': 1}])
    assert out2 == [
        'test [test]\n',
        '    a: 1\n'
    ]


def test_setprocessor():
    rootlogger = _TagTracer()
    log = rootlogger.get("1")
    log2 = log.get("2")
    assert log2.tags == tuple("12")
    out = []
    rootlogger.setprocessor(tuple("12"), lambda *args: out.append(args))
    log("not seen")
    log2("seen")
    assert len(out) == 1
    tags, args = out[0]
    assert "1" in tags
    assert "2" in tags
    assert args == ("seen",)
    l2 = []
    rootlogger.setprocessor("1:2", lambda *args: l2.append(args))
    log2("seen")
    tags, args = l2[0]
    assert args == ("seen",)


def test_setmyprocessor():
    rootlogger = _TagTracer()
    log = rootlogger.get("1")
    log2 = log.get("2")
    out = []
    log2.setmyprocessor(lambda *args: out.append(args))
    log("not seen")
    assert not out
    log2(42)
    assert len(out) == 1
    tags, args = out[0]
    assert "1" in tags
    assert "2" in tags
    assert args == (42,)
