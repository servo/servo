# Copied from the standard library
import collections.abc
import sys
import unittest

import pytest

from exceptiongroup import BaseExceptionGroup, ExceptionGroup


class TestExceptionGroupTypeHierarchy(unittest.TestCase):
    def test_exception_group_types(self):
        self.assertTrue(issubclass(ExceptionGroup, Exception))
        self.assertTrue(issubclass(ExceptionGroup, BaseExceptionGroup))
        self.assertTrue(issubclass(BaseExceptionGroup, BaseException))

    def test_exception_group_is_generic_type(self):
        E = OSError
        self.assertEqual(ExceptionGroup[E].__origin__, ExceptionGroup)
        self.assertEqual(BaseExceptionGroup[E].__origin__, BaseExceptionGroup)


class BadConstructorArgs(unittest.TestCase):
    def test_bad_EG_construction__too_few_args(self):
        if sys.version_info >= (3, 11):
            MSG = (
                r"BaseExceptionGroup.__new__\(\) takes exactly 2 arguments \(1 given\)"
            )
        else:
            MSG = (
                r"__new__\(\) missing 1 required positional argument: "
                r"'_ExceptionGroup__exceptions'"
            )

        with self.assertRaisesRegex(TypeError, MSG):
            ExceptionGroup("no errors")
        with self.assertRaisesRegex(TypeError, MSG):
            ExceptionGroup([ValueError("no msg")])

    def test_bad_EG_construction__too_many_args(self):
        if sys.version_info >= (3, 11):
            MSG = (
                r"BaseExceptionGroup.__new__\(\) takes exactly 2 arguments \(3 given\)"
            )
        else:
            MSG = r"__new__\(\) takes 3 positional arguments but 4 were given"

        with self.assertRaisesRegex(TypeError, MSG):
            ExceptionGroup("eg", [ValueError("too")], [TypeError("many")])

    def test_bad_EG_construction__bad_message(self):
        MSG = "argument 1 must be str, not "
        with self.assertRaisesRegex(TypeError, MSG):
            ExceptionGroup(ValueError(12), SyntaxError("bad syntax"))
        with self.assertRaisesRegex(TypeError, MSG):
            ExceptionGroup(None, [ValueError(12)])

    def test_bad_EG_construction__bad_excs_sequence(self):
        MSG = r"second argument \(exceptions\) must be a sequence"
        with self.assertRaisesRegex(TypeError, MSG):
            ExceptionGroup("errors not sequence", {ValueError(42)})
        with self.assertRaisesRegex(TypeError, MSG):
            ExceptionGroup("eg", None)

        MSG = r"second argument \(exceptions\) must be a non-empty sequence"
        with self.assertRaisesRegex(ValueError, MSG):
            ExceptionGroup("eg", [])

    def test_bad_EG_construction__nested_non_exceptions(self):
        MSG = r"Item [0-9]+ of second argument \(exceptions\) is not an exception"
        with self.assertRaisesRegex(ValueError, MSG):
            ExceptionGroup("expect instance, not type", [OSError])
        with self.assertRaisesRegex(ValueError, MSG):
            ExceptionGroup("bad error", ["not an exception"])


class InstanceCreation(unittest.TestCase):
    def test_EG_wraps_Exceptions__creates_EG(self):
        excs = [ValueError(1), TypeError(2)]
        self.assertIs(type(ExceptionGroup("eg", excs)), ExceptionGroup)

    def test_BEG_wraps_Exceptions__creates_EG(self):
        excs = [ValueError(1), TypeError(2)]
        self.assertIs(type(BaseExceptionGroup("beg", excs)), ExceptionGroup)

    def test_EG_wraps_BaseException__raises_TypeError(self):
        MSG = "Cannot nest BaseExceptions in an ExceptionGroup"
        with self.assertRaisesRegex(TypeError, MSG):
            ExceptionGroup("eg", [ValueError(1), KeyboardInterrupt(2)])

    def test_BEG_wraps_BaseException__creates_BEG(self):
        beg = BaseExceptionGroup("beg", [ValueError(1), KeyboardInterrupt(2)])
        self.assertIs(type(beg), BaseExceptionGroup)

    def test_EG_subclass_wraps_non_base_exceptions(self):
        class MyEG(ExceptionGroup):
            pass

        self.assertIs(type(MyEG("eg", [ValueError(12), TypeError(42)])), MyEG)

    @pytest.mark.skipif(
        sys.version_info[:3] == (3, 11, 0),
        reason="Behavior was made stricter in 3.11.1",
    )
    def test_EG_subclass_does_not_wrap_base_exceptions(self):
        class MyEG(ExceptionGroup):
            pass

        msg = "Cannot nest BaseExceptions in 'MyEG'"
        with self.assertRaisesRegex(TypeError, msg):
            MyEG("eg", [ValueError(12), KeyboardInterrupt(42)])

    @pytest.mark.skipif(
        sys.version_info[:3] == (3, 11, 0),
        reason="Behavior was made stricter in 3.11.1",
    )
    def test_BEG_and_E_subclass_does_not_wrap_base_exceptions(self):
        class MyEG(BaseExceptionGroup, ValueError):
            pass

        msg = "Cannot nest BaseExceptions in 'MyEG'"
        with self.assertRaisesRegex(TypeError, msg):
            MyEG("eg", [ValueError(12), KeyboardInterrupt(42)])


def create_simple_eg():
    excs = []
    try:
        try:
            raise MemoryError("context and cause for ValueError(1)")
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)

    try:
        try:
            raise OSError("context for TypeError")
        except OSError:
            raise TypeError(int)
    except TypeError as e:
        excs.append(e)

    try:
        try:
            raise ImportError("context for ValueError(2)")
        except ImportError:
            raise ValueError(2)
    except ValueError as e:
        excs.append(e)

    try:
        raise ExceptionGroup("simple eg", excs)
    except ExceptionGroup as e:
        return e


class ExceptionGroupFields(unittest.TestCase):
    def test_basics_ExceptionGroup_fields(self):
        eg = create_simple_eg()

        # check msg
        self.assertEqual(eg.message, "simple eg")
        self.assertEqual(eg.args[0], "simple eg")

        # check cause and context
        self.assertIsInstance(eg.exceptions[0], ValueError)
        self.assertIsInstance(eg.exceptions[0].__cause__, MemoryError)
        self.assertIsInstance(eg.exceptions[0].__context__, MemoryError)
        self.assertIsInstance(eg.exceptions[1], TypeError)
        self.assertIsNone(eg.exceptions[1].__cause__)
        self.assertIsInstance(eg.exceptions[1].__context__, OSError)
        self.assertIsInstance(eg.exceptions[2], ValueError)
        self.assertIsNone(eg.exceptions[2].__cause__)
        self.assertIsInstance(eg.exceptions[2].__context__, ImportError)

        # check tracebacks
        line0 = create_simple_eg.__code__.co_firstlineno
        tb_linenos = [line0 + 27, [line0 + 6, line0 + 14, line0 + 22]]
        self.assertEqual(eg.__traceback__.tb_lineno, tb_linenos[0])
        self.assertIsNone(eg.__traceback__.tb_next)
        for i in range(3):
            tb = eg.exceptions[i].__traceback__
            self.assertIsNone(tb.tb_next)
            self.assertEqual(tb.tb_lineno, tb_linenos[1][i])

    def test_fields_are_readonly(self):
        eg = ExceptionGroup("eg", [TypeError(1), OSError(2)])

        self.assertEqual(type(eg.exceptions), tuple)

        eg.message
        with self.assertRaises(AttributeError):
            eg.message = "new msg"

        eg.exceptions
        with self.assertRaises(AttributeError):
            eg.exceptions = [OSError("xyz")]

    def test_notes_is_list_of_strings_if_it_exists(self):
        eg = create_simple_eg()

        note = "This is a happy note for the exception group"
        self.assertFalse(hasattr(eg, "__notes__"))
        eg.add_note(note)
        self.assertEqual(eg.__notes__, [note])

    def test_derive_doesn_copy_notes(self):
        eg = create_simple_eg()
        eg.add_note("hello")
        assert eg.__notes__ == ["hello"]
        eg2 = eg.derive([ValueError()])
        assert not hasattr(eg2, "__notes__")


class ExceptionGroupTestBase(unittest.TestCase):
    def assertMatchesTemplate(self, exc, exc_type, template):
        """Assert that the exception matches the template

        A template describes the shape of exc. If exc is a
        leaf exception (i.e., not an exception group) then
        template is an exception instance that has the
        expected type and args value of exc. If exc is an
        exception group, then template is a list of the
        templates of its nested exceptions.
        """
        if exc_type is not None:
            self.assertIs(type(exc), exc_type)

        if isinstance(exc, BaseExceptionGroup):
            self.assertIsInstance(template, collections.abc.Sequence)
            self.assertEqual(len(exc.exceptions), len(template))
            for e, t in zip(exc.exceptions, template):
                self.assertMatchesTemplate(e, None, t)
        else:
            self.assertIsInstance(template, BaseException)
            self.assertEqual(type(exc), type(template))
            self.assertEqual(exc.args, template.args)


class ExceptionGroupSubgroupTests(ExceptionGroupTestBase):
    def setUp(self):
        self.eg = create_simple_eg()
        self.eg_template = [ValueError(1), TypeError(int), ValueError(2)]

    def test_basics_subgroup_split__bad_arg_type(self):
        bad_args = [
            "bad arg",
            OSError("instance not type"),
            [OSError, TypeError],
            (OSError, 42),
        ]
        for arg in bad_args:
            with self.assertRaises(TypeError):
                self.eg.subgroup(arg)
            with self.assertRaises(TypeError):
                self.eg.split(arg)

    def test_basics_subgroup_by_type__passthrough(self):
        eg = self.eg
        # self.assertIs(eg, eg.subgroup(BaseException))
        # self.assertIs(eg, eg.subgroup(Exception))
        self.assertIs(eg, eg.subgroup(BaseExceptionGroup))
        self.assertIs(eg, eg.subgroup(ExceptionGroup))

    def test_basics_subgroup_by_type__no_match(self):
        self.assertIsNone(self.eg.subgroup(OSError))

    def test_basics_subgroup_by_type__match(self):
        eg = self.eg
        testcases = [
            # (match_type, result_template)
            (ValueError, [ValueError(1), ValueError(2)]),
            (TypeError, [TypeError(int)]),
            ((ValueError, TypeError), self.eg_template),
        ]

        for match_type, template in testcases:
            with self.subTest(match=match_type):
                subeg = eg.subgroup(match_type)
                self.assertEqual(subeg.message, eg.message)
                self.assertMatchesTemplate(subeg, ExceptionGroup, template)

    def test_basics_subgroup_by_predicate__passthrough(self):
        self.assertIs(self.eg, self.eg.subgroup(lambda e: True))

    def test_basics_subgroup_by_predicate__no_match(self):
        self.assertIsNone(self.eg.subgroup(lambda e: False))

    def test_basics_subgroup_by_predicate__match(self):
        eg = self.eg
        testcases = [
            # (match_type, result_template)
            (ValueError, [ValueError(1), ValueError(2)]),
            (TypeError, [TypeError(int)]),
            ((ValueError, TypeError), self.eg_template),
        ]

        for match_type, template in testcases:
            subeg = eg.subgroup(lambda e: isinstance(e, match_type))
            self.assertEqual(subeg.message, eg.message)
            self.assertMatchesTemplate(subeg, ExceptionGroup, template)


class ExceptionGroupSplitTests(ExceptionGroupTestBase):
    def setUp(self):
        self.eg = create_simple_eg()
        self.eg_template = [ValueError(1), TypeError(int), ValueError(2)]

    def test_basics_split_by_type__passthrough(self):
        for E in [BaseException, Exception, BaseExceptionGroup, ExceptionGroup]:
            match, rest = self.eg.split(E)
            self.assertMatchesTemplate(match, ExceptionGroup, self.eg_template)
            self.assertIsNone(rest)

    def test_basics_split_by_type__no_match(self):
        match, rest = self.eg.split(OSError)
        self.assertIsNone(match)
        self.assertMatchesTemplate(rest, ExceptionGroup, self.eg_template)

    def test_basics_split_by_type__match(self):
        eg = self.eg
        VE = ValueError
        TE = TypeError
        testcases = [
            # (matcher, match_template, rest_template)
            (VE, [VE(1), VE(2)], [TE(int)]),
            (TE, [TE(int)], [VE(1), VE(2)]),
            ((VE, TE), self.eg_template, None),
            ((OSError, VE), [VE(1), VE(2)], [TE(int)]),
        ]

        for match_type, match_template, rest_template in testcases:
            match, rest = eg.split(match_type)
            self.assertEqual(match.message, eg.message)
            self.assertMatchesTemplate(match, ExceptionGroup, match_template)
            if rest_template is not None:
                self.assertEqual(rest.message, eg.message)
                self.assertMatchesTemplate(rest, ExceptionGroup, rest_template)
            else:
                self.assertIsNone(rest)

    def test_basics_split_by_predicate__passthrough(self):
        match, rest = self.eg.split(lambda e: True)
        self.assertMatchesTemplate(match, ExceptionGroup, self.eg_template)
        self.assertIsNone(rest)

    def test_basics_split_by_predicate__no_match(self):
        match, rest = self.eg.split(lambda e: False)
        self.assertIsNone(match)
        self.assertMatchesTemplate(rest, ExceptionGroup, self.eg_template)

    def test_basics_split_by_predicate__match(self):
        eg = self.eg
        VE = ValueError
        TE = TypeError
        testcases = [
            # (matcher, match_template, rest_template)
            (VE, [VE(1), VE(2)], [TE(int)]),
            (TE, [TE(int)], [VE(1), VE(2)]),
            ((VE, TE), self.eg_template, None),
        ]

        for match_type, match_template, rest_template in testcases:
            match, rest = eg.split(lambda e: isinstance(e, match_type))
            self.assertEqual(match.message, eg.message)
            self.assertMatchesTemplate(match, ExceptionGroup, match_template)
            if rest_template is not None:
                self.assertEqual(rest.message, eg.message)
                self.assertMatchesTemplate(rest, ExceptionGroup, rest_template)


class DeepRecursionInSplitAndSubgroup(unittest.TestCase):
    def make_deep_eg(self):
        e = TypeError(1)
        for _ in range(10000):
            e = ExceptionGroup("eg", [e])
        return e

    def test_deep_split(self):
        e = self.make_deep_eg()
        with self.assertRaises(RecursionError):
            e.split(TypeError)

    def test_deep_subgroup(self):
        e = self.make_deep_eg()
        with self.assertRaises(RecursionError):
            e.subgroup(TypeError)


def leaf_generator(exc, tbs=None):
    if tbs is None:
        tbs = []
    tbs.append(exc.__traceback__)
    if isinstance(exc, BaseExceptionGroup):
        for e in exc.exceptions:
            yield from leaf_generator(e, tbs)
    else:
        # exc is a leaf exception and its traceback
        # is the concatenation of the traceback
        # segments in tbs
        yield exc, tbs
    tbs.pop()


class LeafGeneratorTest(unittest.TestCase):
    # The leaf_generator is mentioned in PEP 654 as a suggestion
    # on how to iterate over leaf nodes of an EG. Is is also
    # used below as a test utility. So we test it here.

    def test_leaf_generator(self):
        eg = create_simple_eg()

        self.assertSequenceEqual([e for e, _ in leaf_generator(eg)], eg.exceptions)

        for e, tbs in leaf_generator(eg):
            self.assertSequenceEqual(tbs, [eg.__traceback__, e.__traceback__])


def create_nested_eg():
    excs = []
    try:
        try:
            raise TypeError(bytes)
        except TypeError as e:
            raise ExceptionGroup("nested", [e])
    except ExceptionGroup as e:
        excs.append(e)

    try:
        try:
            raise MemoryError("out of memory")
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)

    try:
        raise ExceptionGroup("root", excs)
    except ExceptionGroup as eg:
        return eg


class NestedExceptionGroupBasicsTest(ExceptionGroupTestBase):
    def test_nested_group_matches_template(self):
        eg = create_nested_eg()
        self.assertMatchesTemplate(
            eg, ExceptionGroup, [[TypeError(bytes)], ValueError(1)]
        )

    def test_nested_group_chaining(self):
        eg = create_nested_eg()
        self.assertIsInstance(eg.exceptions[1].__context__, MemoryError)
        self.assertIsInstance(eg.exceptions[1].__cause__, MemoryError)
        self.assertIsInstance(eg.exceptions[0].__context__, TypeError)

    def test_nested_exception_group_tracebacks(self):
        eg = create_nested_eg()

        line0 = create_nested_eg.__code__.co_firstlineno
        for tb, expected in [
            (eg.__traceback__, line0 + 19),
            (eg.exceptions[0].__traceback__, line0 + 6),
            (eg.exceptions[1].__traceback__, line0 + 14),
            (eg.exceptions[0].exceptions[0].__traceback__, line0 + 4),
        ]:
            self.assertEqual(tb.tb_lineno, expected)
            self.assertIsNone(tb.tb_next)

    def test_iteration_full_tracebacks(self):
        eg = create_nested_eg()
        # check that iteration over leaves
        # produces the expected tracebacks
        self.assertEqual(len(list(leaf_generator(eg))), 2)

        line0 = create_nested_eg.__code__.co_firstlineno
        expected_tbs = [[line0 + 19, line0 + 6, line0 + 4], [line0 + 19, line0 + 14]]

        for i, (_, tbs) in enumerate(leaf_generator(eg)):
            self.assertSequenceEqual([tb.tb_lineno for tb in tbs], expected_tbs[i])


class ExceptionGroupSplitTestBase(ExceptionGroupTestBase):
    def split_exception_group(self, eg, types):
        """Split an EG and do some sanity checks on the result"""
        self.assertIsInstance(eg, BaseExceptionGroup)

        match, rest = eg.split(types)
        sg = eg.subgroup(types)

        if match is not None:
            self.assertIsInstance(match, BaseExceptionGroup)
            for e, _ in leaf_generator(match):
                self.assertIsInstance(e, types)

            self.assertIsNotNone(sg)
            self.assertIsInstance(sg, BaseExceptionGroup)
            for e, _ in leaf_generator(sg):
                self.assertIsInstance(e, types)

        if rest is not None:
            self.assertIsInstance(rest, BaseExceptionGroup)

        def leaves(exc):
            return [] if exc is None else [e for e, _ in leaf_generator(exc)]

        # match and subgroup have the same leaves
        self.assertSequenceEqual(leaves(match), leaves(sg))

        match_leaves = leaves(match)
        rest_leaves = leaves(rest)
        # each leaf exception of eg is in exactly one of match and rest
        self.assertEqual(len(leaves(eg)), len(leaves(match)) + len(leaves(rest)))

        for e in leaves(eg):
            self.assertNotEqual(match and e in match_leaves, rest and e in rest_leaves)

        # message, cause and context, traceback and note equal to eg
        for part in [match, rest, sg]:
            if part is not None:
                self.assertEqual(eg.message, part.message)
                self.assertIs(eg.__cause__, part.__cause__)
                self.assertIs(eg.__context__, part.__context__)
                self.assertIs(eg.__traceback__, part.__traceback__)
                self.assertEqual(
                    getattr(eg, "__notes__", None),
                    getattr(part, "__notes__", None),
                )

        def tbs_for_leaf(leaf, eg):
            for e, tbs in leaf_generator(eg):
                if e is leaf:
                    return tbs

        def tb_linenos(tbs):
            return [tb.tb_lineno for tb in tbs if tb]

        # full tracebacks match
        for part in [match, rest, sg]:
            for e in leaves(part):
                self.assertSequenceEqual(
                    tb_linenos(tbs_for_leaf(e, eg)), tb_linenos(tbs_for_leaf(e, part))
                )

        return match, rest


class NestedExceptionGroupSplitTest(ExceptionGroupSplitTestBase):
    def test_split_by_type(self):
        class MyExceptionGroup(ExceptionGroup):
            pass

        def raiseVE(v):
            raise ValueError(v)

        def raiseTE(t):
            raise TypeError(t)

        def nested_group():
            def level1(i):
                excs = []
                for f, arg in [(raiseVE, i), (raiseTE, int), (raiseVE, i + 1)]:
                    try:
                        f(arg)
                    except Exception as e:
                        excs.append(e)
                raise ExceptionGroup("msg1", excs)

            def level2(i):
                excs = []
                for f, arg in [(level1, i), (level1, i + 1), (raiseVE, i + 2)]:
                    try:
                        f(arg)
                    except Exception as e:
                        excs.append(e)
                raise MyExceptionGroup("msg2", excs)

            def level3(i):
                excs = []
                for f, arg in [(level2, i + 1), (raiseVE, i + 2)]:
                    try:
                        f(arg)
                    except Exception as e:
                        excs.append(e)
                raise ExceptionGroup("msg3", excs)

            level3(5)

        try:
            nested_group()
        except ExceptionGroup as e:
            e.add_note(f"the note: {id(e)}")
            eg = e

        eg_template = [
            [
                [ValueError(6), TypeError(int), ValueError(7)],
                [ValueError(7), TypeError(int), ValueError(8)],
                ValueError(8),
            ],
            ValueError(7),
        ]

        valueErrors_template = [
            [
                [ValueError(6), ValueError(7)],
                [ValueError(7), ValueError(8)],
                ValueError(8),
            ],
            ValueError(7),
        ]

        typeErrors_template = [[[TypeError(int)], [TypeError(int)]]]

        self.assertMatchesTemplate(eg, ExceptionGroup, eg_template)

        # Match Nothing
        match, rest = self.split_exception_group(eg, SyntaxError)
        self.assertIsNone(match)
        self.assertMatchesTemplate(rest, ExceptionGroup, eg_template)

        # Match Everything
        match, rest = self.split_exception_group(eg, BaseException)
        self.assertMatchesTemplate(match, ExceptionGroup, eg_template)
        self.assertIsNone(rest)
        match, rest = self.split_exception_group(eg, (ValueError, TypeError))
        self.assertMatchesTemplate(match, ExceptionGroup, eg_template)
        self.assertIsNone(rest)

        # Match ValueErrors
        match, rest = self.split_exception_group(eg, ValueError)
        self.assertMatchesTemplate(match, ExceptionGroup, valueErrors_template)
        self.assertMatchesTemplate(rest, ExceptionGroup, typeErrors_template)

        # Match TypeErrors
        match, rest = self.split_exception_group(eg, (TypeError, SyntaxError))
        self.assertMatchesTemplate(match, ExceptionGroup, typeErrors_template)
        self.assertMatchesTemplate(rest, ExceptionGroup, valueErrors_template)

        # Match ExceptionGroup
        match, rest = eg.split(ExceptionGroup)
        self.assertIs(match, eg)
        self.assertIsNone(rest)

        # Match MyExceptionGroup (ExceptionGroup subclass)
        match, rest = eg.split(MyExceptionGroup)
        self.assertMatchesTemplate(match, ExceptionGroup, [eg_template[0]])
        self.assertMatchesTemplate(rest, ExceptionGroup, [eg_template[1]])

    def test_split_BaseExceptionGroup(self):
        def exc(ex):
            try:
                raise ex
            except BaseException as e:
                return e

        try:
            raise BaseExceptionGroup(
                "beg", [exc(ValueError(1)), exc(KeyboardInterrupt(2))]
            )
        except BaseExceptionGroup as e:
            beg = e

        # Match Nothing
        match, rest = self.split_exception_group(beg, TypeError)
        self.assertIsNone(match)
        self.assertMatchesTemplate(
            rest, BaseExceptionGroup, [ValueError(1), KeyboardInterrupt(2)]
        )

        # Match Everything
        match, rest = self.split_exception_group(beg, (ValueError, KeyboardInterrupt))
        self.assertMatchesTemplate(
            match, BaseExceptionGroup, [ValueError(1), KeyboardInterrupt(2)]
        )
        self.assertIsNone(rest)

        # Match ValueErrors
        match, rest = self.split_exception_group(beg, ValueError)
        self.assertMatchesTemplate(match, ExceptionGroup, [ValueError(1)])
        self.assertMatchesTemplate(rest, BaseExceptionGroup, [KeyboardInterrupt(2)])

        # Match KeyboardInterrupts
        match, rest = self.split_exception_group(beg, KeyboardInterrupt)
        self.assertMatchesTemplate(match, BaseExceptionGroup, [KeyboardInterrupt(2)])
        self.assertMatchesTemplate(rest, ExceptionGroup, [ValueError(1)])


class NestedExceptionGroupSubclassSplitTest(ExceptionGroupSplitTestBase):
    def test_split_ExceptionGroup_subclass_no_derive_no_new_override(self):
        class EG(ExceptionGroup):
            pass

        try:
            try:
                try:
                    raise TypeError(2)
                except TypeError as te:
                    raise EG("nested", [te])
            except EG as nested:
                try:
                    raise ValueError(1)
                except ValueError as ve:
                    raise EG("eg", [ve, nested])
        except EG as e:
            eg = e

        self.assertMatchesTemplate(eg, EG, [ValueError(1), [TypeError(2)]])

        # Match Nothing
        match, rest = self.split_exception_group(eg, OSError)
        self.assertIsNone(match)
        self.assertMatchesTemplate(
            rest, ExceptionGroup, [ValueError(1), [TypeError(2)]]
        )

        # Match Everything
        match, rest = self.split_exception_group(eg, (ValueError, TypeError))
        self.assertMatchesTemplate(
            match, ExceptionGroup, [ValueError(1), [TypeError(2)]]
        )
        self.assertIsNone(rest)

        # Match ValueErrors
        match, rest = self.split_exception_group(eg, ValueError)
        self.assertMatchesTemplate(match, ExceptionGroup, [ValueError(1)])
        self.assertMatchesTemplate(rest, ExceptionGroup, [[TypeError(2)]])

        # Match TypeErrors
        match, rest = self.split_exception_group(eg, TypeError)
        self.assertMatchesTemplate(match, ExceptionGroup, [[TypeError(2)]])
        self.assertMatchesTemplate(rest, ExceptionGroup, [ValueError(1)])

    def test_split_BaseExceptionGroup_subclass_no_derive_new_override(self):
        class EG(BaseExceptionGroup):
            def __new__(cls, message, excs, unused):
                # The "unused" arg is here to show that split() doesn't call
                # the actual class constructor from the default derive()
                # implementation (it would fail on unused arg if so because
                # it assumes the BaseExceptionGroup.__new__ signature).
                return super().__new__(cls, message, excs)

        try:
            raise EG("eg", [ValueError(1), KeyboardInterrupt(2)], "unused")
        except EG as e:
            eg = e

        self.assertMatchesTemplate(eg, EG, [ValueError(1), KeyboardInterrupt(2)])

        # Match Nothing
        match, rest = self.split_exception_group(eg, OSError)
        self.assertIsNone(match)
        self.assertMatchesTemplate(
            rest, BaseExceptionGroup, [ValueError(1), KeyboardInterrupt(2)]
        )

        # Match Everything
        match, rest = self.split_exception_group(eg, (ValueError, KeyboardInterrupt))
        self.assertMatchesTemplate(
            match, BaseExceptionGroup, [ValueError(1), KeyboardInterrupt(2)]
        )
        self.assertIsNone(rest)

        # Match ValueErrors
        match, rest = self.split_exception_group(eg, ValueError)
        self.assertMatchesTemplate(match, ExceptionGroup, [ValueError(1)])
        self.assertMatchesTemplate(rest, BaseExceptionGroup, [KeyboardInterrupt(2)])

        # Match KeyboardInterrupt
        match, rest = self.split_exception_group(eg, KeyboardInterrupt)
        self.assertMatchesTemplate(match, BaseExceptionGroup, [KeyboardInterrupt(2)])
        self.assertMatchesTemplate(rest, ExceptionGroup, [ValueError(1)])

    def test_split_ExceptionGroup_subclass_derive_and_new_overrides(self):
        class EG(ExceptionGroup):
            def __new__(cls, message, excs, code):
                obj = super().__new__(cls, message, excs)
                obj.code = code
                return obj

            def derive(self, excs):
                return EG(self.message, excs, self.code)

        try:
            try:
                try:
                    raise TypeError(2)
                except TypeError as te:
                    raise EG("nested", [te], 101)
            except EG as nested:
                try:
                    raise ValueError(1)
                except ValueError as ve:
                    raise EG("eg", [ve, nested], 42)
        except EG as e:
            e.add_note("hello")
            eg = e

        self.assertMatchesTemplate(eg, EG, [ValueError(1), [TypeError(2)]])

        # Match Nothing
        match, rest = self.split_exception_group(eg, OSError)
        self.assertIsNone(match)
        self.assertMatchesTemplate(rest, EG, [ValueError(1), [TypeError(2)]])
        self.assertEqual(rest.code, 42)
        self.assertEqual(rest.exceptions[1].code, 101)
        self.assertEqual(rest.__notes__, ["hello"])

        # Match Everything
        match, rest = self.split_exception_group(eg, (ValueError, TypeError))
        self.assertMatchesTemplate(match, EG, [ValueError(1), [TypeError(2)]])
        self.assertEqual(match.code, 42)
        self.assertEqual(match.exceptions[1].code, 101)
        self.assertEqual(match.__notes__, ["hello"])
        self.assertIsNone(rest)

        # Match ValueErrors
        match, rest = self.split_exception_group(eg, ValueError)
        self.assertMatchesTemplate(match, EG, [ValueError(1)])
        self.assertEqual(match.code, 42)
        self.assertEqual(match.__notes__, ["hello"])
        self.assertMatchesTemplate(rest, EG, [[TypeError(2)]])
        self.assertEqual(rest.code, 42)
        self.assertEqual(rest.exceptions[0].code, 101)
        self.assertEqual(rest.__notes__, ["hello"])

        # Match TypeErrors
        match, rest = self.split_exception_group(eg, TypeError)
        self.assertMatchesTemplate(match, EG, [[TypeError(2)]])
        self.assertEqual(match.code, 42)
        self.assertEqual(match.exceptions[0].code, 101)
        self.assertEqual(match.__notes__, ["hello"])
        self.assertMatchesTemplate(rest, EG, [ValueError(1)])
        self.assertEqual(rest.code, 42)
        self.assertEqual(rest.__notes__, ["hello"])


def test_repr():
    group = BaseExceptionGroup("foo", [ValueError(1), KeyboardInterrupt()])
    assert repr(group) == (
        "BaseExceptionGroup('foo', [ValueError(1), KeyboardInterrupt()])"
    )

    group = ExceptionGroup("foo", [ValueError(1), RuntimeError("bar")])
    assert repr(group) == "ExceptionGroup('foo', [ValueError(1), RuntimeError('bar')])"
