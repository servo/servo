# mypy: allow-untyped-defs

import os
from urllib.parse import urljoin, urlsplit
from collections import namedtuple, defaultdict, deque
from math import ceil
from typing import Any, Callable, ClassVar, Dict, List

from .wptmanifest import serialize
from .wptmanifest.node import (DataNode, ConditionalNode, BinaryExpressionNode,
                              BinaryOperatorNode, NumberNode, StringNode, VariableNode,
                              ValueNode, UnaryExpressionNode, UnaryOperatorNode,
                              ListNode)
from .wptmanifest.backends import conditional
from .wptmanifest.backends.conditional import ManifestItem

from . import expected
from . import expectedtree

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
new results are known, update is called to compute the new
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


class UpdateProperties:
    def __init__(self, manifest, **kwargs):
        self._manifest = manifest
        self._classes = kwargs

    def __getattr__(self, name):
        if name in self._classes:
            rv = self._classes[name](self._manifest)
            setattr(self, name, rv)
            return rv
        raise AttributeError

    def __contains__(self, name):
        return name in self._classes

    def __iter__(self):
        for name in self._classes.keys():
            yield getattr(self, name)


class ExpectedManifest(ManifestItem):
    def __init__(self, node, test_path, url_base, run_info_properties,
                 update_intermittent=False, remove_intermittent=False):
        """Object representing all the tests in a particular manifest

        :param node: AST Node associated with this object. If this is None,
                     a new AST is created to associate with this manifest.
        :param test_path: Path of the test file associated with this manifest.
        :param url_base: Base url for serving the tests in this manifest.
        :param run_info_properties: Tuple of ([property name],
                                              {property_name: [dependent property]})
                                    The first part lists run_info properties
                                    that are always used in the update, the second
                                    maps property names to additional properties that
                                    can be considered if we already have a condition on
                                    the key property e.g. {"foo": ["bar"]} means that
                                    we consider making conditions on bar only after we
                                    already made one on foo.
        :param update_intermittent: When True, intermittent statuses will be recorded
                                    as `expected` in the test metadata.
        :param: remove_intermittent: When True, old intermittent statuses will be removed
                                    if no longer intermittent. This is only relevant if
                                    `update_intermittent` is also True, because if False,
                                    the metadata will simply update one `expected`status.
        """
        if node is None:
            node = DataNode(None)
        ManifestItem.__init__(self, node)
        self.child_map = {}
        self.test_path = test_path
        self.url_base = url_base
        assert self.url_base is not None
        self._modified = False
        self.run_info_properties = run_info_properties
        self.update_intermittent = update_intermittent
        self.remove_intermittent = remove_intermittent
        self.update_properties = UpdateProperties(self, **{
            "lsan": LsanUpdate,
            "leak_object": LeakObjectUpdate,
            "leak_threshold": LeakThresholdUpdate,
        })

    @property
    def modified(self):
        if self._modified:
            return True
        return any(item.modified for item in self.children)

    @modified.setter
    def modified(self, value):
        self._modified = value

    def append(self, child):
        ManifestItem.append(self, child)
        if child.id in self.child_map:
            print("Warning: Duplicate heading %s" % child.id)
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
        return urljoin(self.url_base,
                       "/".join(self.test_path.split(os.path.sep)))

    def set_lsan(self, run_info, result):
        """Set the result of the test in a particular run

        :param run_info: Dictionary of run_info parameters corresponding
                         to this run
        :param result: Lsan violations detected"""
        self.update_properties.lsan.set(run_info, result)

    def set_leak_object(self, run_info, result):
        """Set the result of the test in a particular run

        :param run_info: Dictionary of run_info parameters corresponding
                         to this run
        :param result: Leaked objects deletec"""
        self.update_properties.leak_object.set(run_info, result)

    def set_leak_threshold(self, run_info, result):
        """Set the result of the test in a particular run

        :param run_info: Dictionary of run_info parameters corresponding
                         to this run
        :param result: Total number of bytes leaked"""
        self.update_properties.leak_threshold.set(run_info, result)

    def update(self, full_update, disable_intermittent):
        for prop_update in self.update_properties:
            prop_update.update(full_update,
                               disable_intermittent)


class TestNode(ManifestItem):
    def __init__(self, node):
        """Tree node associated with a particular test in a manifest

        :param node: AST node associated with the test"""

        ManifestItem.__init__(self, node)
        self.subtests = {}
        self._from_file = True
        self.new_disabled = False
        self.has_result = False
        self.modified = False
        self.update_properties = UpdateProperties(
            self,
            expected=ExpectedUpdate,
            max_asserts=MaxAssertsUpdate,
            min_asserts=MinAssertsUpdate
        )

    @classmethod
    def create(cls, test_id):
        """Create a TestNode corresponding to a given test

        :param test_type: The type of the test
        :param test_id: The id of the test"""
        name = test_id[len(urlsplit(test_id).path.rsplit("/", 1)[0]) + 1:]
        node = DataNode(name)
        self = cls(node)

        self._from_file = False
        return self

    @property
    def is_empty(self):
        ignore_keys = {"type"}
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
        return urljoin(self.parent.url, self.name)

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
        self.update_properties.expected.set(run_info, result)

    def set_asserts(self, run_info, count):
        """Set the assert count of a test

        """
        self.update_properties.min_asserts.set(run_info, count)
        self.update_properties.max_asserts.set(run_info, count)

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

    def update(self, full_update, disable_intermittent):
        for prop_update in self.update_properties:
            prop_update.update(full_update,
                               disable_intermittent)


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


def build_conditional_tree(_, run_info_properties, results):
    properties, dependent_props = run_info_properties
    return expectedtree.build_tree(properties, dependent_props, results)


def build_unconditional_tree(_, run_info_properties, results):
    root = expectedtree.Node(None, None)
    for run_info, values in results.items():
        for value, count in values.items():
            root.result_values[value] += count
        root.run_info.add(run_info)
    return root


class PropertyUpdate:
    property_name = None  # type: ClassVar[str]
    cls_default_value = None  # type: ClassVar[Any]
    value_type = None  # type: ClassVar[type]
    # property_builder is a class variable set to either build_conditional_tree
    # or build_unconditional_tree. TODO: Make this type stricter when those
    # methods are annotated.
    property_builder = None  # type: ClassVar[Callable[..., Any]]

    def __init__(self, node):
        self.node = node
        self.default_value = self.cls_default_value
        self.has_result = False
        self.results = defaultdict(lambda: defaultdict(int))
        self.update_intermittent = self.node.root.update_intermittent
        self.remove_intermittent = self.node.root.remove_intermittent

    def run_info_by_condition(self, run_info_index, conditions):
        run_info_by_condition = defaultdict(list)
        # A condition might match 0 or more run_info values
        run_infos = run_info_index.keys()
        for cond in conditions:
            for run_info in run_infos:
                if cond(run_info):
                    run_info_by_condition[cond].append(run_info)

        return run_info_by_condition

    def set(self, run_info, value):
        self.has_result = True
        self.node.has_result = True
        self.check_default(value)
        value = self.from_result_value(value)
        self.results[run_info][value] += 1

    def check_default(self, result):
        return

    def from_result_value(self, value):
        """Convert a value from a test result into the internal format"""
        return value

    def from_ini_value(self, value):
        """Convert a value from an ini file into the internal format"""
        if self.value_type:
            return self.value_type(value)
        return value

    def to_ini_value(self, value):
        """Convert a value from the internal format to the ini file format"""
        return str(value)

    def updated_value(self, current, new):
        """Given a single current value and a set of observed new values,
        compute an updated value for the property"""
        return new

    @property
    def unconditional_value(self):
        try:
            unconditional_value = self.from_ini_value(
                self.node.get(self.property_name))
        except KeyError:
            unconditional_value = self.default_value
        return unconditional_value

    def update(self,
               full_update=False,
               disable_intermittent=None):
        """Update the underlying manifest AST for this test based on all the
        added results.

        This will update existing conditionals if they got the same result in
        all matching runs in the updated results, will delete existing conditionals
        that get more than one different result in the updated run, and add new
        conditionals for anything that doesn't match an existing conditional.

        Conditionals not matched by any added result are not changed.

        When `disable_intermittent` is not None, disable any test that shows multiple
        unexpected results for the same set of parameters.
        """
        if not self.has_result:
            return

        property_tree = self.property_builder(self.node.root.run_info_properties,
                                              self.results)

        conditions, errors = self.update_conditions(property_tree,
                                                    full_update)

        for e in errors:
            if disable_intermittent:
                condition = e.cond.children[0] if e.cond else None
                msg = disable_intermittent if isinstance(disable_intermittent, str) else "unstable"
                self.node.set("disabled", msg, condition)
                self.node.new_disabled = True
            else:
                msg = "Conflicting metadata values for %s" % (
                    self.node.root.test_path)
                if e.cond:
                    msg += ": %s" % serialize(e.cond).strip()
                print(msg)

        # If all the values match remove all conditionals
        # This handles the case where we update a number of existing conditions and they
        # all end up looking like the post-update default.
        new_default = self.default_value
        if conditions and conditions[-1][0] is None:
            new_default = conditions[-1][1]
        if all(condition[1] == new_default for condition in conditions):
            conditions = [(None, new_default)]

        # Don't set the default to the class default
        if (conditions and
            conditions[-1][0] is None and
            conditions[-1][1] == self.default_value):
            self.node.modified = True
            conditions = conditions[:-1]

        if self.node.modified:
            self.node.clear(self.property_name)

            for condition, value in conditions:
                self.node.set(self.property_name,
                              self.to_ini_value(value),
                              condition)

    def update_conditions(self,
                          property_tree,
                          full_update):
        # This is complicated because the expected behaviour is complex
        # The complexity arises from the fact that there are two ways of running
        # the tool, with a full set of runs (full_update=True) or with partial metadata
        # (full_update=False). In the case of a full update things are relatively simple:
        # * All existing conditionals are ignored, with the exception of conditionals that
        #   depend on variables not used by the updater, which are retained as-is
        # * All created conditionals are independent of each other (i.e. order isn't
        #   important in the created conditionals)
        # In the case where we don't have a full set of runs, the expected behaviour
        # is much less clear. This is of course the common case for when a developer
        # runs the test on their own machine. In this case the assumptions above are untrue
        # * The existing conditions may be required to handle other platforms
        # * The order of the conditions may be important, since we don't know if they overlap
        #   e.g. `if os == linux and version == 18.04` overlaps with `if (os != win)`.
        # So in the case we have a full set of runs, the process is pretty simple:
        # * Generate the conditionals for the property_tree
        # * Pick the most common value as the default and add only those conditions
        #   not matching the default
        # In the case where we have a partial set of runs, things are more complex
        # and more best-effort
        # * For each existing conditional, see if it matches any of the run info we
        #   have. In cases where it does match, record the new results
        # * Where all the new results match, update the right hand side of that
        #   conditional, otherwise remove it
        # * If this leaves nothing existing, then proceed as with the full update
        # * Otherwise add conditionals for the run_info that doesn't match any
        #   remaining conditions
        prev_default = None

        current_conditions = self.node.get_conditions(self.property_name)

        # Ignore the current default value
        if current_conditions and current_conditions[-1].condition_node is None:
            self.node.modified = True
            prev_default = current_conditions[-1].value
            current_conditions = current_conditions[:-1]

        # If there aren't any current conditions, or there is just a default
        # value for all run_info, proceed as for a full update
        if not current_conditions:
            return self._update_conditions_full(property_tree,
                                                prev_default=prev_default)

        conditions = []
        errors = []

        run_info_index = {run_info: node
                          for node in property_tree
                          for run_info in node.run_info}

        node_by_run_info = {run_info: node
                            for (run_info, node) in run_info_index.items()
                            if node.result_values}

        run_info_by_condition = self.run_info_by_condition(run_info_index,
                                                           current_conditions)

        run_info_with_condition = set()

        if full_update:
            # Even for a full update we need to keep hand-written conditions not
            # using the properties we've specified and not matching any run_info
            top_level_props, dependent_props = self.node.root.run_info_properties
            update_properties = set(top_level_props)
            for item in dependent_props.values():
                update_properties |= set(item)
            for condition in current_conditions:
                if (not condition.variables.issubset(update_properties) and
                    not run_info_by_condition[condition]):
                    conditions.append((condition.condition_node,
                                       self.from_ini_value(condition.value)))

            new_conditions, errors = self._update_conditions_full(property_tree,
                                                                  prev_default=prev_default)
            conditions.extend(new_conditions)
            return conditions, errors

        # Retain existing conditions if they match the updated values
        for condition in current_conditions:
            # All run_info that isn't handled by some previous condition
            all_run_infos_condition = run_info_by_condition[condition]
            run_infos = {item for item in all_run_infos_condition
                         if item not in run_info_with_condition}

            if not run_infos:
                # Retain existing conditions that don't match anything in the update
                conditions.append((condition.condition_node,
                                   self.from_ini_value(condition.value)))
                continue

            # Set of nodes in the updated tree that match the same run_info values as the
            # current existing node
            nodes = [node_by_run_info[run_info] for run_info in run_infos
                     if run_info in node_by_run_info]
            # If all the values are the same, update the value
            if nodes and all(set(node.result_values.keys()) == set(nodes[0].result_values.keys()) for node in nodes):
                current_value = self.from_ini_value(condition.value)
                try:
                    new_value = self.updated_value(current_value,
                                                   nodes[0].result_values)
                except ConditionError as e:
                    errors.append(e)
                    continue
                if new_value != current_value:
                    self.node.modified = True
                conditions.append((condition.condition_node, new_value))
                run_info_with_condition |= set(run_infos)
            else:
                # Don't append this condition
                self.node.modified = True

        new_conditions, new_errors = self.build_tree_conditions(property_tree,
                                                                run_info_with_condition,
                                                                prev_default)
        if new_conditions:
            self.node.modified = True

        conditions.extend(new_conditions)
        errors.extend(new_errors)

        return conditions, errors

    def _update_conditions_full(self,
                                property_tree,
                                prev_default=None):
        self.node.modified = True
        conditions, errors = self.build_tree_conditions(property_tree,
                                                        set(),
                                                        prev_default)

        return conditions, errors

    def build_tree_conditions(self,
                              property_tree,
                              run_info_with_condition,
                              prev_default=None):
        conditions = []
        errors = []

        value_count = defaultdict(int)

        def to_count_value(v):
            if v is None:
                return v
            # Need to count the values in a hashable type
            count_value = self.to_ini_value(v)
            if isinstance(count_value, list):
                count_value = tuple(count_value)
            return count_value


        queue = deque([(property_tree, [])])
        while queue:
            node, parents = queue.popleft()
            parents_and_self = parents + [node]
            if node.result_values and any(run_info not in run_info_with_condition
                                          for run_info in node.run_info):
                prop_set = [(item.prop, item.value) for item in parents_and_self if item.prop]
                value = node.result_values
                error = None
                if parents:
                    try:
                        value = self.updated_value(None, value)
                    except ConditionError:
                        expr = make_expr(prop_set, value)
                        error = ConditionError(expr)
                    else:
                        expr = make_expr(prop_set, value)
                else:
                    # The root node needs special handling
                    expr = None
                    try:
                        value = self.updated_value(self.unconditional_value,
                                                   value)
                    except ConditionError:
                        error = ConditionError(expr)
                        # If we got an error for the root node, re-add the previous
                        # default value
                        if prev_default:
                            conditions.append((None, prev_default))
                if error is None:
                    count_value = to_count_value(value)
                    value_count[count_value] += len(node.run_info)

                if error is None:
                    conditions.append((expr, value))
                else:
                    errors.append(error)

            for child in node.children:
                queue.append((child, parents_and_self))

        conditions = conditions[::-1]

        # If we haven't set a default condition, add one and remove all the conditions
        # with the same value
        if value_count and (not conditions or conditions[-1][0] is not None):
            # Sort in order of occurence, prioritising values that match the class default
            # or the previous default
            cls_default = to_count_value(self.default_value)
            prev_default = to_count_value(prev_default)
            commonest_value = max(value_count, key=lambda x:(value_count.get(x),
                                                             x == cls_default,
                                                             x == prev_default))
            if isinstance(commonest_value, tuple):
                commonest_value = list(commonest_value)
            commonest_value = self.from_ini_value(commonest_value)
            conditions = [item for item in conditions if item[1] != commonest_value]
            conditions.append((None, commonest_value))

        return conditions, errors


class ExpectedUpdate(PropertyUpdate):
    property_name = "expected"
    property_builder = build_conditional_tree

    def check_default(self, result):
        if self.default_value is not None:
            assert self.default_value == result.default_expected
        else:
            self.default_value = result.default_expected

    def from_result_value(self, result):
        # When we are updating intermittents, we need to keep a record of any existing
        # intermittents to pass on when building the property tree and matching statuses and
        # intermittents to the correct run info -  this is so we can add them back into the
        # metadata aligned with the right conditions, unless specified not to with
        # self.remove_intermittent.
        # The (status, known_intermittent) tuple is counted when the property tree is built, but
        # the count value only applies to the first item in the tuple, the status from that run,
        # when passed to `updated_value`.
        if (not self.update_intermittent or
            self.remove_intermittent or
            not result.known_intermittent):
            return result.status
        return result.status + result.known_intermittent

    def to_ini_value(self, value):
        if isinstance(value, (list, tuple)):
            return [str(item) for item in value]
        return str(value)

    def updated_value(self, current, new):
        if len(new) > 1 and not self.update_intermittent and not isinstance(current, list):
            raise ConditionError

        counts = {}
        for status, count in new.items():
            if isinstance(status, tuple):
                counts[status[0]] = count
                counts.update({intermittent: 0 for intermittent in status[1:] if intermittent not in counts})
            else:
                counts[status] = count

        if not (self.update_intermittent or isinstance(current, list)):
            return list(counts)[0]

        # Reorder statuses first based on counts, then based on status priority if there are ties.
        # Counts with 0 are considered intermittent.
        statuses = ["OK", "PASS", "FAIL", "ERROR", "TIMEOUT", "CRASH"]
        status_priority = {value: i for i, value in enumerate(statuses)}
        sorted_new = sorted(counts.items(), key=lambda x:(-1 * x[1],
                                                        status_priority.get(x[0],
                                                        len(status_priority))))
        expected = []
        for status, count in sorted_new:
            # If we are not removing existing recorded intermittents, with a count of 0,
            # add them in to expected.
            if count > 0 or not self.remove_intermittent:
                expected.append(status)

        # If the new intermittent is a subset of the existing one, just use the existing one
        # This prevents frequent flip-flopping of results between e.g. [OK, TIMEOUT] and
        # [TIMEOUT, OK]
        if current and set(expected).issubset(set(current)):
            return current

        if self.update_intermittent:
            if len(expected) == 1:
                return expected[0]
            return expected

        # If we are not updating intermittents, return the status with the highest occurence.
        return expected[0]


class MaxAssertsUpdate(PropertyUpdate):
    """For asserts we always update the default value and never add new conditionals.
    The value we set as the default is the maximum the current default or one more than the
    number of asserts we saw in any configuration."""

    property_name = "max-asserts"
    cls_default_value = 0
    value_type = int
    property_builder = build_unconditional_tree

    def updated_value(self, current, new):
        if any(item > current for item in new):
            return max(new) + 1
        return current


class MinAssertsUpdate(PropertyUpdate):
    property_name = "min-asserts"
    cls_default_value = 0
    value_type = int
    property_builder = build_unconditional_tree

    def updated_value(self, current, new):
        if any(item < current for item in new):
            rv = min(new) - 1
        else:
            rv = current
        return max(rv, 0)


class AppendOnlyListUpdate(PropertyUpdate):
    cls_default_value = []  # type: ClassVar[List[str]]
    property_builder = build_unconditional_tree

    def updated_value(self, current, new):
        if current is None:
            rv = set()
        else:
            rv = set(current)

        for item in new:
            if item is None:
                continue
            elif isinstance(item, str):
                rv.add(item)
            else:
                rv |= item

        return sorted(rv)


class LsanUpdate(AppendOnlyListUpdate):
    property_name = "lsan-allowed"
    property_builder = build_unconditional_tree

    def from_result_value(self, result):
        # If we have an allowed_match that matched, return None
        # This value is ignored later (because it matches the default)
        # We do that because then if we allow a failure in foo/__dir__.ini
        # we don't want to update foo/bar/__dir__.ini with the same rule
        if result[1]:
            return None
        # Otherwise return the topmost stack frame
        # TODO: there is probably some improvement to be made by looking for a "better" stack frame
        return result[0][0]

    def to_ini_value(self, value):
        return value


class LeakObjectUpdate(AppendOnlyListUpdate):
    property_name = "leak-allowed"
    property_builder = build_unconditional_tree

    def from_result_value(self, result):
        # If we have an allowed_match that matched, return None
        if result[1]:
            return None
        # Otherwise return the process/object name
        return result[0]


class LeakThresholdUpdate(PropertyUpdate):
    property_name = "leak-threshold"
    cls_default_value = {}  # type: ClassVar[Dict[str, int]]
    property_builder = build_unconditional_tree

    def from_result_value(self, result):
        return result

    def to_ini_value(self, data):
        return ["%s:%s" % item for item in sorted(data.items())]

    def from_ini_value(self, data):
        rv = {}
        for item in data:
            key, value = item.split(":", 1)
            rv[key] = int(float(value))
        return rv

    def updated_value(self, current, new):
        if current:
            rv = current.copy()
        else:
            rv = {}
        for process, leaked_bytes, threshold in new:
            # If the value is less than the threshold but there isn't
            # an old value we must have inherited the threshold from
            # a parent ini file so don't any anything to this one
            if process not in rv and leaked_bytes < threshold:
                continue
            if leaked_bytes > rv.get(process, 0):
                # Round up to nearest 50 kb
                boundary = 50 * 1024
                rv[process] = int(boundary * ceil(float(leaked_bytes) / boundary))
        return rv


def make_expr(prop_set, rhs):
    """Create an AST that returns the value ``status`` given all the
    properties in prop_set match.

    :param prop_set: tuple of (property name, value) pairs for each
                     property in this expression and the value it must match
    :param status: Status on RHS when all the given properties match
    """
    root = ConditionalNode()

    assert len(prop_set) > 0

    expressions = []
    for prop, value in prop_set:
        if value not in (True, False):
            expressions.append(
                BinaryExpressionNode(
                    BinaryOperatorNode("=="),
                    VariableNode(prop),
                    make_node(value)))
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
    rhs_node = make_value_node(rhs)
    root.append(rhs_node)

    return root


def make_node(value):
    if isinstance(value, (int, float,)):
        node = NumberNode(value)
    elif isinstance(value, str):
        node = StringNode(str(value))
    elif hasattr(value, "__iter__"):
        node = ListNode()
        for item in value:
            node.append(make_node(item))
    return node


def make_value_node(value):
    if isinstance(value, (int, float,)):
        node = ValueNode(value)
    elif isinstance(value, str):
        node = ValueNode(str(value))
    elif hasattr(value, "__iter__"):
        node = ListNode()
        for item in value:
            node.append(make_value_node(item))
    else:
        raise ValueError("Don't know how to convert %s into node" % type(value))
    return node


def get_manifest(metadata_root, test_path, url_base, run_info_properties, update_intermittent, remove_intermittent):
    """Get the ExpectedManifest for a particular test path, or None if there is no
    metadata stored for that test path.

    :param metadata_root: Absolute path to the root of the metadata directory
    :param test_path: Path to the test(s) relative to the test root
    :param url_base: Base url for serving the tests in this manifest"""
    manifest_path = expected.expected_path(metadata_root, test_path)
    try:
        with open(manifest_path, "rb") as f:
            rv = compile(f, test_path, url_base,
                         run_info_properties, update_intermittent, remove_intermittent)
    except OSError:
        return None
    return rv


def compile(manifest_file, test_path, url_base, run_info_properties, update_intermittent, remove_intermittent):
    return conditional.compile(manifest_file,
                               data_cls_getter=data_cls_getter,
                               test_path=test_path,
                               url_base=url_base,
                               run_info_properties=run_info_properties,
                               update_intermittent=update_intermittent,
                               remove_intermittent=remove_intermittent)
