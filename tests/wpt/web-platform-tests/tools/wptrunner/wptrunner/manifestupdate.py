import itertools
import os
import urlparse
from collections import namedtuple, defaultdict

from wptmanifest.node import (DataNode, ConditionalNode, BinaryExpressionNode,
                              BinaryOperatorNode, VariableNode, StringNode, NumberNode,
                              UnaryExpressionNode, UnaryOperatorNode, KeyValueNode)
from wptmanifest.backends import conditional
from wptmanifest.backends.conditional import ManifestItem

import expected

"""Manifest structure used to update the expected results of a test

Each manifest file is represented by an ExpectedManifest that has one
or more TestNode children, one per test in the manifest.  Each
TestNode has zero or more SubtestNode children, one for each known
subtest of the test.

In these representations, conditionals expressions in the manifest are
not evaluated upfront but stored as python functions to be evaluated
at runtime.

When a result for a test is to be updated set_result on the
[Sub]TestNode is called to store the new result, alongside the
existing conditional that result's run info matched, if any. Once all
new results are known, coalesce_expected is called to compute the new
set of results and conditionals. The AST of the underlying parsed manifest
is updated with the changes, and the result is serialised to a file.
"""


class ConditionError(Exception):
    def __init__(self, cond=None):
        self.cond = cond


class UpdateError(Exception):
    pass


Value = namedtuple("Value", ["run_info", "value"])


def data_cls_getter(output_node, visited_node):
    # visited_node is intentionally unused
    if output_node is None:
        return ExpectedManifest
    elif isinstance(output_node, ExpectedManifest):
        return TestNode
    elif isinstance(output_node, TestNode):
        return SubtestNode
    else:
        raise ValueError


class ExpectedManifest(ManifestItem):
    def __init__(self, node, test_path=None, url_base=None, property_order=None,
                 boolean_properties=None):
        """Object representing all the tests in a particular manifest

        :param node: AST Node associated with this object. If this is None,
                     a new AST is created to associate with this manifest.
        :param test_path: Path of the test file associated with this manifest.
        :param url_base: Base url for serving the tests in this manifest.
        :param property_order: List of properties to use in expectation metadata
                               from most to least significant.
        :param boolean_properties: Set of properties in property_order that should
                                   be treated as boolean.
        """
        if node is None:
            node = DataNode(None)
        ManifestItem.__init__(self, node)
        self.child_map = {}
        self.test_path = test_path
        self.url_base = url_base
        assert self.url_base is not None
        self.modified = False
        self.boolean_properties = boolean_properties
        self.property_order = property_order
        self.update_properties = {
            "lsan": LsanUpdate(self),
        }

    def append(self, child):
        ManifestItem.append(self, child)
        if child.id in self.child_map:
            print "Warning: Duplicate heading %s" % child.id
        self.child_map[child.id] = child

    def _remove_child(self, child):
        del self.child_map[child.id]
        ManifestItem._remove_child(self, child)

    def get_test(self, test_id):
        """Return a TestNode by test id, or None if no test matches

        :param test_id: The id of the test to look up"""

        return self.child_map.get(test_id)

    def has_test(self, test_id):
        """Boolean indicating whether the current test has a known child test
        with id test id

        :param test_id: The id of the test to look up"""

        return test_id in self.child_map

    @property
    def url(self):
        return urlparse.urljoin(self.url_base,
                                "/".join(self.test_path.split(os.path.sep)))

    def set_lsan(self, run_info, result):
        """Set the result of the test in a particular run

        :param run_info: Dictionary of run_info parameters corresponding
                         to this run
        :param result: Lsan violations detected"""

        self.update_properties["lsan"].set(run_info, result)

    def coalesce_properties(self, stability):
        for prop_update in self.update_properties.itervalues():
            prop_update.coalesce(stability)


class TestNode(ManifestItem):
    def __init__(self, node):
        """Tree node associated with a particular test in a manifest

        :param node: AST node associated with the test"""

        ManifestItem.__init__(self, node)
        self.subtests = {}
        self._from_file = True
        self.new_disabled = False
        self.update_properties = {
            "expected": ExpectedUpdate(self),
            "max-asserts": MaxAssertsUpdate(self),
            "min-asserts": MinAssertsUpdate(self)
        }

    @classmethod
    def create(cls, test_id):
        """Create a TestNode corresponding to a given test

        :param test_type: The type of the test
        :param test_id: The id of the test"""

        url = test_id
        name = url.rsplit("/", 1)[1]
        node = DataNode(name)
        self = cls(node)

        self._from_file = False
        return self

    @property
    def is_empty(self):
        ignore_keys = set(["type"])
        if set(self._data.keys()) - ignore_keys:
            return False
        return all(child.is_empty for child in self.children)

    @property
    def test_type(self):
        """The type of the test represented by this TestNode"""
        return self.get("type", None)

    @property
    def id(self):
        """The id of the test represented by this TestNode"""
        return urlparse.urljoin(self.parent.url, self.name)

    def disabled(self, run_info):
        """Boolean indicating whether this test is disabled when run in an
        environment with the given run_info

        :param run_info: Dictionary of run_info parameters"""

        return self.get("disabled", run_info) is not None

    def set_result(self, run_info, result):
        """Set the result of the test in a particular run

        :param run_info: Dictionary of run_info parameters corresponding
                         to this run
        :param result: Status of the test in this run"""

        self.update_properties["expected"].set(run_info, result)

    def set_asserts(self, run_info, count):
        """Set the assert count of a test

        """
        self.update_properties["min-asserts"].set(run_info, count)
        self.update_properties["max-asserts"].set(run_info, count)

    def _add_key_value(self, node, values):
        ManifestItem._add_key_value(self, node, values)
        if node.data in self.update_properties:
            new_updated = []
            self.update_properties[node.data].updated = new_updated
            for value in values:
                new_updated.append((value, []))

    def clear(self, key):
        """Clear all the expected data for this test and all of its subtests"""

        self.updated = []
        if key in self._data:
            for child in self.node.children:
                if (isinstance(child, KeyValueNode) and
                    child.data == key):
                    child.remove()
                    del self._data[key]
                    break

        for subtest in self.subtests.itervalues():
            subtest.clear(key)

    def append(self, node):
        child = ManifestItem.append(self, node)
        self.subtests[child.name] = child

    def get_subtest(self, name):
        """Return a SubtestNode corresponding to a particular subtest of
        the current test, creating a new one if no subtest with that name
        already exists.

        :param name: Name of the subtest"""

        if name in self.subtests:
            return self.subtests[name]
        else:
            subtest = SubtestNode.create(name)
            self.append(subtest)
            return subtest

    def coalesce_properties(self, stability):
        for prop_update in self.update_properties.itervalues():
            prop_update.coalesce(stability)


class SubtestNode(TestNode):
    def __init__(self, node):
        assert isinstance(node, DataNode)
        TestNode.__init__(self, node)

    @classmethod
    def create(cls, name):
        node = DataNode(name)
        self = cls(node)
        return self

    @property
    def is_empty(self):
        if self._data:
            return False
        return True


class PropertyUpdate(object):
    property_name = None
    cls_default_value = None
    value_type = None

    def __init__(self, node):
        self.node = node
        self.updated = []
        self.new = []
        self.default_value = self.cls_default_value

    def set(self, run_info, in_value):
        self.check_default(in_value)
        value = self.get_value(in_value)

        # Add this result to the list of results satisfying
        # any condition in the list of updated results it matches
        for (cond, values) in self.updated:
            if cond(run_info):
                values.append(Value(run_info, value))
                if value != cond.value_as(self.value_type):
                    self.node.root.modified = True
                break
        else:
            # We didn't find a previous value for this
            self.new.append(Value(run_info, value))
            self.node.root.modified = True

    def check_default(self, result):
        return

    def get_value(self, in_value):
        return in_value

    def coalesce(self, stability=None):
        """Update the underlying manifest AST for this test based on all the
        added results.

        This will update existing conditionals if they got the same result in
        all matching runs in the updated results, will delete existing conditionals
        that get more than one different result in the updated run, and add new
        conditionals for anything that doesn't match an existing conditional.

        Conditionals not matched by any added result are not changed.

        When `stability` is not None, disable any test that shows multiple
        unexpected results for the same set of parameters.
        """

        try:
            unconditional_value = self.node.get(self.property_name)
            if self.value_type:
                unconditional_value = self.value_type(unconditional_value)
        except KeyError:
            unconditional_value = self.default_value

        for conditional_value, results in self.updated:
            if not results:
                # The conditional didn't match anything in these runs so leave it alone
                pass
            elif all(results[0].value == result.value for result in results):
                # All the new values for this conditional matched, so update the node
                result = results[0]
                if (result.value == unconditional_value and
                    conditional_value.condition_node is not None):
                    if self.property_name in self.node:
                        self.node.remove_value(self.property_name, conditional_value)
                else:
                    conditional_value.value = self.update_value(conditional_value.value_as(self.value_type),
                                                                result.value)
            elif conditional_value.condition_node is not None:
                # Blow away the existing condition and rebuild from scratch
                # This isn't sure to work if we have a conditional later that matches
                # these values too, but we can hope, verify that we get the results
                # we expect, and if not let a human sort it out
                self.node.remove_value(self.property_name, conditional_value)
                self.new.extend(results)
            elif conditional_value.condition_node is None:
                self.new.extend(result for result in results
                                if result.value != unconditional_value)

        # It is an invariant that nothing in new matches an existing
        # condition except for the default condition
        if self.new:
            update_default, new_default_value = self.update_default()
            if update_default:
                if new_default_value != self.default_value:
                    self.node.set(self.property_name,
                                  self.update_value(unconditional_value, new_default_value),
                                  condition=None)
            else:
                try:
                    self.add_new(unconditional_value, stability)
                except UpdateError as e:
                    print("%s for %s, cannot update %s" % (e, self.node.root.test_path,
                                                           self.property_name))

        # Remove cases where the value matches the default
        if (self.property_name in self.node._data and
            len(self.node._data[self.property_name]) > 0 and
            self.node._data[self.property_name][-1].condition_node is None and
            self.node._data[self.property_name][-1].value_as(self.value_type) == self.default_value):

            self.node.remove_value(self.property_name, self.node._data[self.property_name][-1])

        # Remove empty properties
        if (self.property_name in self.node._data and len(self.node._data[self.property_name]) == 0):
            for child in self.node.children:
                if (isinstance(child, KeyValueNode) and child.data == self.property_name):
                    child.remove()
                    break

    def update_default(self):
        """Get the updated default value for the property (i.e. the one chosen when no conditions match).

        :returns: (update, new_default_value) where updated is a bool indicating whether the property
                  should be updated, and new_default_value is the value to set if it should."""
        raise NotImplementedError

    def add_new(self, unconditional_value, stability):
        """Add new conditional values for the property.

        Subclasses need not implement this if they only ever update the default value."""
        raise NotImplementedError

    def update_value(self, old_value, new_value):
        """Get a value to set on the property, given its previous value and the new value from logs.

        By default this just returns the new value, but overriding is useful in cases
        where we want the new value to be some function of both old and new e.g. max(old_value, new_value)"""
        return new_value


class ExpectedUpdate(PropertyUpdate):
    property_name = "expected"

    def check_default(self, result):
        if self.default_value is not None:
            assert self.default_value == result.default_expected
        else:
            self.default_value = result.default_expected

    def get_value(self, in_value):
        return in_value.status

    def update_default(self):
        update_default = all(self.new[0].value == result.value
                             for result in self.new) and not self.updated
        new_value = self.new[0].value
        return update_default, new_value

    def add_new(self, unconditional_value, stability):
        try:
            conditionals = group_conditionals(
                self.new,
                property_order=self.node.root.property_order,
                boolean_properties=self.node.root.boolean_properties)
        except ConditionError as e:
            if stability is not None:
                self.node.set("disabled", stability or "unstable", e.cond.children[0])
                self.node.new_disabled = True
            else:
                raise UpdateError("Conflicting metadata values")
        for conditional_node, value in conditionals:
            if value != unconditional_value:
                self.node.set(self.property_name, value, condition=conditional_node.children[0])


class MaxAssertsUpdate(PropertyUpdate):
    property_name = "max-asserts"
    cls_default_value = 0
    value_type = int

    def update_value(self, old_value, new_value):
        new_value = self.value_type(new_value)
        if old_value is not None:
            old_value = self.value_type(old_value)
        if old_value is not None and old_value < new_value:
            return new_value + 1
        if old_value is None:
            return new_value + 1
        return old_value

    def update_default(self):
        """For asserts we always update the default value and never add new conditionals.
        The value we set as the default is the maximum the current default or one more than the
        number of asserts we saw in any configuration."""
        # Current values
        values = []
        current_default = None
        if self.property_name in self.node._data:
            current_default = [item for item in
                               self.node._data[self.property_name]
                               if item.condition_node is None]
            if current_default:
                values.append(int(current_default[0].value))
        values.extend(item.value for item in self.new)
        values.extend(item.value for item in
                      itertools.chain.from_iterable(results for _, results in self.updated))
        new_value = max(values)
        return True, new_value


class MinAssertsUpdate(PropertyUpdate):
    property_name = "min-asserts"
    cls_default_value = 0
    value_type = int

    def update_value(self, old_value, new_value):
        new_value = self.value_type(new_value)
        if old_value is not None:
            old_value = self.value_type(old_value)
        if old_value is not None and new_value < old_value:
            return 0
        if old_value is None:
            # If we are getting some asserts for the first time, set the minimum to 0
            return new_value
        return old_value

    def update_default(self):
        """For asserts we always update the default value and never add new conditionals.
        This is either set to the current value or one less than the number of asserts
        we saw, whichever is lower."""
        values = []
        current_default = None
        if self.property_name in self.node._data:
            current_default = [item for item in
                               self.node._data[self.property_name]
                               if item.condition_node is None]
        if current_default:
            values.append(current_default[0].value_as(self.value_type))
        values.extend(max(0, item.value) for item in self.new)
        values.extend(max(0, item.value) for item in
                      itertools.chain.from_iterable(results for _, results in self.updated))
        new_value = min(values)
        return True, new_value


class LsanUpdate(PropertyUpdate):
    property_name = "lsan-allowed"
    cls_default_value = None

    def get_value(self, result):
        # If we have an allowed_match that matched, return None
        # This value is ignored later (because it matches the default)
        # We do that because then if we allow a failure in foo/__dir__.ini
        # we don't want to update foo/bar/__dir__.ini with the same rule
        if result[1]:
            return None
        # Otherwise return the topmost stack frame
        # TODO: there is probably some improvement to be made by looking for a "better" stack frame
        return result[0][0]

    def update_value(self, old_value, new_value):
        if isinstance(new_value, (str, unicode)):
            new_value = {new_value}
        else:
            new_value = set(new_value)
        if old_value is None:
            old_value = set()
        old_value = set(old_value)
        return sorted((old_value | new_value) - {None})

    def update_default(self):
        current_default = None
        if self.property_name in self.node._data:
            current_default = [item for item in
                               self.node._data[self.property_name]
                               if item.condition_node is None]
        if current_default:
            current_default = current_default[0].value
        new_values = [item.value for item in self.new]
        new_value = self.update_value(current_default, new_values)
        return True, new_value if new_value else None


def group_conditionals(values, property_order=None, boolean_properties=None):
    """Given a list of Value objects, return a list of
    (conditional_node, status) pairs representing the conditional
    expressions that are required to match each status

    :param values: List of Values
    :param property_order: List of properties to use in expectation metadata
                           from most to least significant.
    :param boolean_properties: Set of properties in property_order that should
                               be treated as boolean."""

    by_property = defaultdict(set)
    for run_info, value in values:
        for prop_name, prop_value in run_info.iteritems():
            by_property[(prop_name, prop_value)].add(value)

    if property_order is None:
        property_order = ["debug", "os", "version", "processor", "bits"]

    if boolean_properties is None:
        boolean_properties = set(["debug"])
    else:
        boolean_properties = set(boolean_properties)

    # If we have more than one value, remove any properties that are common
    # for all the values
    if len(values) > 1:
        for key, statuses in by_property.copy().iteritems():
            if len(statuses) == len(values):
                del by_property[key]
        if not by_property:
            raise ConditionError

    properties = set(item[0] for item in by_property.iterkeys())
    include_props = []

    for prop in property_order:
        if prop in properties:
            include_props.append(prop)

    conditions = {}

    for run_info, value in values:
        prop_set = tuple((prop, run_info[prop]) for prop in include_props)
        if prop_set in conditions:
            if conditions[prop_set][1] != value:
                # A prop_set contains contradictory results
                raise ConditionError(make_expr(prop_set, value, boolean_properties))
            continue

        expr = make_expr(prop_set, value, boolean_properties=boolean_properties)
        conditions[prop_set] = (expr, value)

    return conditions.values()


def make_expr(prop_set, rhs, boolean_properties=None):
    """Create an AST that returns the value ``status`` given all the
    properties in prop_set match.

    :param prop_set: tuple of (property name, value) pairs for each
                     property in this expression and the value it must match
    :param status: Status on RHS when all the given properties match
    :param boolean_properties: Set of properties in property_order that should
                               be treated as boolean.
    """
    root = ConditionalNode()

    assert len(prop_set) > 0

    expressions = []
    for prop, value in prop_set:
        number_types = (int, float, long)
        value_cls = (NumberNode
                     if type(value) in number_types
                     else StringNode)
        if prop not in boolean_properties:
            expressions.append(
                BinaryExpressionNode(
                    BinaryOperatorNode("=="),
                    VariableNode(prop),
                    value_cls(unicode(value))
                ))
        else:
            if value:
                expressions.append(VariableNode(prop))
            else:
                expressions.append(
                    UnaryExpressionNode(
                        UnaryOperatorNode("not"),
                        VariableNode(prop)
                    ))
    if len(expressions) > 1:
        prev = expressions[-1]
        for curr in reversed(expressions[:-1]):
            node = BinaryExpressionNode(
                BinaryOperatorNode("and"),
                curr,
                prev)
            prev = node
    else:
        node = expressions[0]

    root.append(node)
    if type(rhs) in number_types:
        rhs_node = NumberNode(rhs)
    else:
        rhs_node = StringNode(rhs)
    root.append(rhs_node)

    return root


def get_manifest(metadata_root, test_path, url_base, property_order=None,
                 boolean_properties=None):
    """Get the ExpectedManifest for a particular test path, or None if there is no
    metadata stored for that test path.

    :param metadata_root: Absolute path to the root of the metadata directory
    :param test_path: Path to the test(s) relative to the test root
    :param url_base: Base url for serving the tests in this manifest
    :param property_order: List of properties to use in expectation metadata
                           from most to least significant.
    :param boolean_properties: Set of properties in property_order that should
                               be treated as boolean."""
    manifest_path = expected.expected_path(metadata_root, test_path)
    try:
        with open(manifest_path) as f:
            return compile(f, test_path, url_base, property_order=property_order,
                           boolean_properties=boolean_properties)
    except IOError:
        return None


def compile(manifest_file, test_path, url_base, property_order=None,
            boolean_properties=None):
    return conditional.compile(manifest_file,
                               data_cls_getter=data_cls_getter,
                               test_path=test_path,
                               url_base=url_base,
                               property_order=property_order,
                               boolean_properties=boolean_properties)
