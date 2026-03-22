# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import unittest

from mozlog.logtypes import Any, Dict, Int, List, TestList as TypeTestList, Tuple, Unicode


class TestContainerTypes(unittest.TestCase):
    def test_dict_type_basic(self):
        d = Dict("name")
        with self.assertRaises(ValueError):
            d({"foo": "bar"})

        d = Dict(Any, "name")
        d({"foo": "bar"})  # doesn't raise

    def test_dict_type_with_dictionary_item_type(self):
        d = Dict({Int: Int}, "name")
        with self.assertRaises(ValueError):
            d({"foo": 1})

        with self.assertRaises(ValueError):
            d({1: "foo"})

        d({1: 2})  # doesn't raise

    def test_dict_type_with_recursive_item_types(self):
        d = Dict(Dict({Unicode: List(Int)}), "name")
        with self.assertRaises(ValueError):
            d({"foo": "bar"})

        with self.assertRaises(ValueError):
            d({"foo": {"bar": "baz"}})

        with self.assertRaises(ValueError):
            d({"foo": {"bar": ["baz"]}})

        d({"foo": {"bar": [1]}})  # doesn't raise

    def test_list_type_basic(self):
        l = List("name")
        with self.assertRaises(ValueError):
            l(["foo"])

        l = List(Any, "name")
        l(["foo", 1])  # doesn't raise

    def test_list_type_with_recursive_item_types(self):
        l = List(Dict(List(Tuple((Unicode, Int)))), "name")
        with self.assertRaises(ValueError):
            l(["foo"])

        with self.assertRaises(ValueError):
            l([{"foo": "bar"}])

        with self.assertRaises(ValueError):
            l([{"foo": ["bar"]}])

        l([{"foo": [("bar", 1)]}])  # doesn't raise

    def test_tuple_type_basic(self):
        t = Tuple("name")
        with self.assertRaises(ValueError):
            t((1,))

        t = Tuple(Any, "name")
        t((1,))  # doesn't raise

    def test_tuple_type_with_tuple_item_type(self):
        t = Tuple((Unicode, Int))
        with self.assertRaises(ValueError):
            t(("foo", "bar"))

        t(("foo", 1))  # doesn't raise

    def test_tuple_type_with_recursive_item_types(self):
        t = Tuple((Dict(List(Any)), List(Dict(Any)), Unicode), "name")
        with self.assertRaises(ValueError):
            t(({"foo": "bar"}, [{"foo": "bar"}], "foo"))

        with self.assertRaises(ValueError):
            t(({"foo": ["bar"]}, ["foo"], "foo"))

        t(({"foo": ["bar"]}, [{"foo": "bar"}], "foo"))  # doesn't raise


class TestDataTypes(unittest.TestCase):
    def test_test_list(self):
        t = TypeTestList("name")
        with self.assertRaises(ValueError):
            t("foo")

        with self.assertRaises(ValueError):
            t({"foo": 1})

        d1 = t({"default": ["bar"]})  # doesn't raise
        d2 = t(["bar"])  # doesn't raise

        self.assertLessEqual(d1.items(), d2.items())


if __name__ == "__main__":
    import mozunit
    mozunit.main()
