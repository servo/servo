# -*- coding: utf-8 -*-
import sys
import textwrap

import _pytest.assertion as plugin
import _pytest._code
import py
import pytest
from _pytest.assertion import reinterpret
from _pytest.assertion import util

PY3 = sys.version_info >= (3, 0)


@pytest.fixture
def mock_config():
    class Config(object):
        verbose = False
        def getoption(self, name):
            if name == 'verbose':
                return self.verbose
            raise KeyError('Not mocked out: %s' % name)
    return Config()


def interpret(expr):
    return reinterpret.reinterpret(expr, _pytest._code.Frame(sys._getframe(1)))

class TestBinReprIntegration:

    def test_pytest_assertrepr_compare_called(self, testdir):
        testdir.makeconftest("""
            l = []
            def pytest_assertrepr_compare(op, left, right):
                l.append((op, left, right))
            def pytest_funcarg__l(request):
                return l
        """)
        testdir.makepyfile("""
            def test_hello():
                assert 0 == 1
            def test_check(l):
                assert l == [("==", 0, 1)]
        """)
        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines([
            "*test_hello*FAIL*",
            "*test_check*PASS*",
        ])

def callequal(left, right, verbose=False):
    config = mock_config()
    config.verbose = verbose
    return plugin.pytest_assertrepr_compare(config, '==', left, right)


class TestAssert_reprcompare:
    def test_different_types(self):
        assert callequal([0, 1], 'foo') is None

    def test_summary(self):
        summary = callequal([0, 1], [0, 2])[0]
        assert len(summary) < 65

    def test_text_diff(self):
        diff = callequal('spam', 'eggs')[1:]
        assert '- spam' in diff
        assert '+ eggs' in diff

    def test_text_skipping(self):
        lines = callequal('a'*50 + 'spam', 'a'*50 + 'eggs')
        assert 'Skipping' in lines[1]
        for line in lines:
            assert 'a'*50 not in line

    def test_text_skipping_verbose(self):
        lines = callequal('a'*50 + 'spam', 'a'*50 + 'eggs', verbose=True)
        assert '- ' + 'a'*50 + 'spam' in lines
        assert '+ ' + 'a'*50 + 'eggs' in lines

    def test_multiline_text_diff(self):
        left = 'foo\nspam\nbar'
        right = 'foo\neggs\nbar'
        diff = callequal(left, right)
        assert '- spam' in diff
        assert '+ eggs' in diff

    def test_list(self):
        expl = callequal([0, 1], [0, 2])
        assert len(expl) > 1

    @pytest.mark.parametrize(
        ['left', 'right', 'expected'], [
            ([0, 1], [0, 2], """
                Full diff:
                - [0, 1]
                ?     ^
                + [0, 2]
                ?     ^
            """),
            ({0: 1}, {0: 2}, """
                Full diff:
                - {0: 1}
                ?     ^
                + {0: 2}
                ?     ^
            """),
            (set([0, 1]), set([0, 2]), """
                Full diff:
                - set([0, 1])
                ?         ^
                + set([0, 2])
                ?         ^
            """ if not PY3 else """
                Full diff:
                - {0, 1}
                ?     ^
                + {0, 2}
                ?     ^
            """)
        ]
    )
    def test_iterable_full_diff(self, left, right, expected):
        """Test the full diff assertion failure explanation.

        When verbose is False, then just a -v notice to get the diff is rendered,
        when verbose is True, then ndiff of the pprint is returned.
        """
        expl = callequal(left, right, verbose=False)
        assert expl[-1] == 'Use -v to get the full diff'
        expl = '\n'.join(callequal(left, right, verbose=True))
        assert expl.endswith(textwrap.dedent(expected).strip())

    def test_list_different_lenghts(self):
        expl = callequal([0, 1], [0, 1, 2])
        assert len(expl) > 1
        expl = callequal([0, 1, 2], [0, 1])
        assert len(expl) > 1

    def test_dict(self):
        expl = callequal({'a': 0}, {'a': 1})
        assert len(expl) > 1

    def test_dict_omitting(self):
        lines = callequal({'a': 0, 'b': 1}, {'a': 1, 'b': 1})
        assert lines[1].startswith('Omitting 1 identical item')
        assert 'Common items' not in lines
        for line in lines[1:]:
            assert 'b' not in line

    def test_dict_omitting_verbose(self):
        lines = callequal({'a': 0, 'b': 1}, {'a': 1, 'b': 1}, verbose=True)
        assert lines[1].startswith('Common items:')
        assert 'Omitting' not in lines[1]
        assert lines[2] == "{'b': 1}"

    def test_set(self):
        expl = callequal(set([0, 1]), set([0, 2]))
        assert len(expl) > 1

    def test_frozenzet(self):
        expl = callequal(frozenset([0, 1]), set([0, 2]))
        assert len(expl) > 1

    def test_Sequence(self):
        col = py.builtin._tryimport(
            "collections.abc",
            "collections",
            "sys")
        if not hasattr(col, "MutableSequence"):
            pytest.skip("cannot import MutableSequence")
        MutableSequence = col.MutableSequence

        class TestSequence(MutableSequence):  # works with a Sequence subclass
            def __init__(self, iterable):
                self.elements = list(iterable)

            def __getitem__(self, item):
                return self.elements[item]

            def __len__(self):
                return len(self.elements)

            def __setitem__(self, item, value):
                pass

            def __delitem__(self, item):
                pass

            def insert(self, item, index):
                pass

        expl = callequal(TestSequence([0, 1]), list([0, 2]))
        assert len(expl) > 1

    def test_list_tuples(self):
        expl = callequal([], [(1,2)])
        assert len(expl) > 1
        expl = callequal([(1,2)], [])
        assert len(expl) > 1

    def test_list_bad_repr(self):
        class A:
            def __repr__(self):
                raise ValueError(42)
        expl = callequal([], [A()])
        assert 'ValueError' in "".join(expl)
        expl = callequal({}, {'1': A()})
        assert 'faulty' in "".join(expl)

    def test_one_repr_empty(self):
        """
        the faulty empty string repr did trigger
        a unbound local error in _diff_text
        """
        class A(str):
            def __repr__(self):
                return ''
        expl = callequal(A(), '')
        assert not expl

    def test_repr_no_exc(self):
        expl = ' '.join(callequal('foo', 'bar'))
        assert 'raised in repr()' not in expl

    def test_unicode(self):
        left = py.builtin._totext('£€', 'utf-8')
        right = py.builtin._totext('£', 'utf-8')
        expl = callequal(left, right)
        assert expl[0] == py.builtin._totext("'£€' == '£'", 'utf-8')
        assert expl[1] == py.builtin._totext('- £€', 'utf-8')
        assert expl[2] == py.builtin._totext('+ £', 'utf-8')

    def test_nonascii_text(self):
        """
        :issue: 877
        non ascii python2 str caused a UnicodeDecodeError
        """
        class A(str):
            def __repr__(self):
                return '\xff'
        expl = callequal(A(), '1')
        assert expl

    def test_format_nonascii_explanation(self):
        assert util.format_explanation('λ')

    def test_mojibake(self):
        # issue 429
        left = 'e'
        right = '\xc3\xa9'
        if not isinstance(left, py.builtin.bytes):
            left = py.builtin.bytes(left, 'utf-8')
            right = py.builtin.bytes(right, 'utf-8')
        expl = callequal(left, right)
        for line in expl:
            assert isinstance(line, py.builtin.text)
        msg = py.builtin._totext('\n').join(expl)
        assert msg


class TestFormatExplanation:

    def test_special_chars_full(self, testdir):
        # Issue 453, for the bug this would raise IndexError
        testdir.makepyfile("""
            def test_foo():
                assert '\\n}' == ''
        """)
        result = testdir.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines([
            "*AssertionError*",
        ])

    def test_fmt_simple(self):
        expl = 'assert foo'
        assert util.format_explanation(expl) == 'assert foo'

    def test_fmt_where(self):
        expl = '\n'.join(['assert 1',
                          '{1 = foo',
                          '} == 2'])
        res = '\n'.join(['assert 1 == 2',
                         ' +  where 1 = foo'])
        assert util.format_explanation(expl) == res

    def test_fmt_and(self):
        expl = '\n'.join(['assert 1',
                          '{1 = foo',
                          '} == 2',
                          '{2 = bar',
                          '}'])
        res = '\n'.join(['assert 1 == 2',
                         ' +  where 1 = foo',
                         ' +  and   2 = bar'])
        assert util.format_explanation(expl) == res

    def test_fmt_where_nested(self):
        expl = '\n'.join(['assert 1',
                          '{1 = foo',
                          '{foo = bar',
                          '}',
                          '} == 2'])
        res = '\n'.join(['assert 1 == 2',
                         ' +  where 1 = foo',
                         ' +    where foo = bar'])
        assert util.format_explanation(expl) == res

    def test_fmt_newline(self):
        expl = '\n'.join(['assert "foo" == "bar"',
                          '~- foo',
                          '~+ bar'])
        res = '\n'.join(['assert "foo" == "bar"',
                         '  - foo',
                         '  + bar'])
        assert util.format_explanation(expl) == res

    def test_fmt_newline_escaped(self):
        expl = '\n'.join(['assert foo == bar',
                          'baz'])
        res = 'assert foo == bar\\nbaz'
        assert util.format_explanation(expl) == res

    def test_fmt_newline_before_where(self):
        expl = '\n'.join(['the assertion message here',
                          '>assert 1',
                          '{1 = foo',
                          '} == 2',
                          '{2 = bar',
                          '}'])
        res = '\n'.join(['the assertion message here',
                         'assert 1 == 2',
                         ' +  where 1 = foo',
                         ' +  and   2 = bar'])
        assert util.format_explanation(expl) == res

    def test_fmt_multi_newline_before_where(self):
        expl = '\n'.join(['the assertion',
                          '~message here',
                          '>assert 1',
                          '{1 = foo',
                          '} == 2',
                          '{2 = bar',
                          '}'])
        res = '\n'.join(['the assertion',
                         '  message here',
                         'assert 1 == 2',
                         ' +  where 1 = foo',
                         ' +  and   2 = bar'])
        assert util.format_explanation(expl) == res


def test_python25_compile_issue257(testdir):
    testdir.makepyfile("""
        def test_rewritten():
            assert 1 == 2
        # some comment
    """)
    result = testdir.runpytest()
    assert result.ret == 1
    result.stdout.fnmatch_lines("""
            *E*assert 1 == 2*
            *1 failed*
    """)

def test_rewritten(testdir):
    testdir.makepyfile("""
        def test_rewritten():
            assert "@py_builtins" in globals()
    """)
    assert testdir.runpytest().ret == 0

def test_reprcompare_notin(mock_config):
    detail = plugin.pytest_assertrepr_compare(
        mock_config, 'not in', 'foo', 'aaafoobbb')[1:]
    assert detail == ["'foo' is contained here:", '  aaafoobbb', '?    +++']

def test_pytest_assertrepr_compare_integration(testdir):
    testdir.makepyfile("""
        def test_hello():
            x = set(range(100))
            y = x.copy()
            y.remove(50)
            assert x == y
    """)
    result = testdir.runpytest()
    result.stdout.fnmatch_lines([
        "*def test_hello():*",
        "*assert x == y*",
        "*E*Extra items*left*",
        "*E*50*",
    ])

def test_sequence_comparison_uses_repr(testdir):
    testdir.makepyfile("""
        def test_hello():
            x = set("hello x")
            y = set("hello y")
            assert x == y
    """)
    result = testdir.runpytest()
    result.stdout.fnmatch_lines([
        "*def test_hello():*",
        "*assert x == y*",
        "*E*Extra items*left*",
        "*E*'x'*",
        "*E*Extra items*right*",
        "*E*'y'*",
    ])


def test_assert_compare_truncate_longmessage(monkeypatch, testdir):
    testdir.makepyfile(r"""
        def test_long():
            a = list(range(200))
            b = a[::2]
            a = '\n'.join(map(str, a))
            b = '\n'.join(map(str, b))
            assert a == b
    """)
    monkeypatch.delenv('CI', raising=False)

    result = testdir.runpytest()
    # without -vv, truncate the message showing a few diff lines only
    result.stdout.fnmatch_lines([
        "*- 1",
        "*- 3",
        "*- 5",
        "*- 7",
        "*truncated (191 more lines)*use*-vv*",
    ])


    result = testdir.runpytest('-vv')
    result.stdout.fnmatch_lines([
        "*- 197",
    ])

    monkeypatch.setenv('CI', '1')
    result = testdir.runpytest()
    result.stdout.fnmatch_lines([
        "*- 197",
    ])


def test_assertrepr_loaded_per_dir(testdir):
    testdir.makepyfile(test_base=['def test_base(): assert 1 == 2'])
    a = testdir.mkdir('a')
    a_test = a.join('test_a.py')
    a_test.write('def test_a(): assert 1 == 2')
    a_conftest = a.join('conftest.py')
    a_conftest.write('def pytest_assertrepr_compare(): return ["summary a"]')
    b = testdir.mkdir('b')
    b_test = b.join('test_b.py')
    b_test.write('def test_b(): assert 1 == 2')
    b_conftest = b.join('conftest.py')
    b_conftest.write('def pytest_assertrepr_compare(): return ["summary b"]')
    result = testdir.runpytest()
    result.stdout.fnmatch_lines([
            '*def test_base():*',
            '*E*assert 1 == 2*',
            '*def test_a():*',
            '*E*assert summary a*',
            '*def test_b():*',
            '*E*assert summary b*'])


def test_assertion_options(testdir):
    testdir.makepyfile("""
        def test_hello():
            x = 3
            assert x == 4
    """)
    result = testdir.runpytest()
    assert "3 == 4" in result.stdout.str()
    off_options = (("--no-assert",),
                   ("--nomagic",),
                   ("--no-assert", "--nomagic"),
                   ("--assert=plain",),
                   ("--assert=plain", "--no-assert"),
                   ("--assert=plain", "--nomagic"),
                   ("--assert=plain", "--no-assert", "--nomagic"))
    for opt in off_options:
        result = testdir.runpytest_subprocess(*opt)
        assert "3 == 4" not in result.stdout.str()

def test_old_assert_mode(testdir):
    testdir.makepyfile("""
        def test_in_old_mode():
            assert "@py_builtins" not in globals()
    """)
    result = testdir.runpytest_subprocess("--assert=reinterp")
    assert result.ret == 0

def test_triple_quoted_string_issue113(testdir):
    testdir.makepyfile("""
        def test_hello():
            assert "" == '''
    '''""")
    result = testdir.runpytest("--fulltrace")
    result.stdout.fnmatch_lines([
        "*1 failed*",
    ])
    assert 'SyntaxError' not in result.stdout.str()

def test_traceback_failure(testdir):
    p1 = testdir.makepyfile("""
        def g():
            return 2
        def f(x):
            assert x == g()
        def test_onefails():
            f(3)
    """)
    result = testdir.runpytest(p1, "--tb=long")
    result.stdout.fnmatch_lines([
        "*test_traceback_failure.py F",
        "====* FAILURES *====",
        "____*____",
        "",
        "    def test_onefails():",
        ">       f(3)",
        "",
        "*test_*.py:6: ",
        "_ _ _ *",
        #"",
        "    def f(x):",
        ">       assert x == g()",
        "E       assert 3 == 2",
        "E        +  where 2 = g()",
        "",
        "*test_traceback_failure.py:4: AssertionError"
    ])

    result = testdir.runpytest(p1) # "auto"
    result.stdout.fnmatch_lines([
        "*test_traceback_failure.py F",
        "====* FAILURES *====",
        "____*____",
        "",
        "    def test_onefails():",
        ">       f(3)",
        "",
        "*test_*.py:6: ",
        "",
        "    def f(x):",
        ">       assert x == g()",
        "E       assert 3 == 2",
        "E        +  where 2 = g()",
        "",
        "*test_traceback_failure.py:4: AssertionError"
    ])

@pytest.mark.skipif("'__pypy__' in sys.builtin_module_names or sys.platform.startswith('java')" )
def test_warn_missing(testdir):
    testdir.makepyfile("")
    result = testdir.run(sys.executable, "-OO", "-m", "pytest", "-h")
    result.stderr.fnmatch_lines([
        "*WARNING*assert statements are not executed*",
    ])
    result = testdir.run(sys.executable, "-OO", "-m", "pytest", "--no-assert")
    result.stderr.fnmatch_lines([
        "*WARNING*assert statements are not executed*",
    ])

def test_recursion_source_decode(testdir):
    testdir.makepyfile("""
        def test_something():
            pass
    """)
    testdir.makeini("""
        [pytest]
        python_files = *.py
    """)
    result = testdir.runpytest("--collect-only")
    result.stdout.fnmatch_lines("""
        <Module*>
    """)

def test_AssertionError_message(testdir):
    testdir.makepyfile("""
        def test_hello():
            x,y = 1,2
            assert 0, (x,y)
    """)
    result = testdir.runpytest()
    result.stdout.fnmatch_lines("""
        *def test_hello*
        *assert 0, (x,y)*
        *AssertionError: (1, 2)*
    """)

@pytest.mark.skipif(PY3, reason='This bug does not exist on PY3')
def test_set_with_unsortable_elements():
    # issue #718
    class UnsortableKey(object):
        def __init__(self, name):
            self.name = name

        def __lt__(self, other):
            raise RuntimeError()

        def __repr__(self):
            return 'repr({0})'.format(self.name)

        def __eq__(self, other):
            return self.name == other.name

        def __hash__(self):
            return hash(self.name)

    left_set = set(UnsortableKey(str(i)) for i in range(1, 3))
    right_set = set(UnsortableKey(str(i)) for i in range(2, 4))
    expl = callequal(left_set, right_set, verbose=True)
    # skip first line because it contains the "construction" of the set, which does not have a guaranteed order
    expl = expl[1:]
    dedent = textwrap.dedent("""
        Extra items in the left set:
        repr(1)
        Extra items in the right set:
        repr(3)
        Full diff (fallback to calling repr on each item):
        - repr(1)
        repr(2)
        + repr(3)
    """).strip()
    assert '\n'.join(expl) == dedent
