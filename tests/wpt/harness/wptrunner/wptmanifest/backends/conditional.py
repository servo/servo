# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import operator

from ..node import NodeVisitor, DataNode, ConditionalNode, KeyValueNode, ValueNode
from ..parser import parse


class ConditionalValue(object):
    def __init__(self, node, condition_func):
        self.node = node
        self.condition_func = condition_func
        if isinstance(node, ConditionalNode):
            assert len(node.children) == 2
            self.condition_node = self.node.children[0]
            self.value_node = self.node.children[1]
        else:
            assert isinstance(node, ValueNode)
            self.condition_node = None
            self.value_node = self.node

    @property
    def value(self):
        return self.value_node.data

    @value.setter
    def value(self, value):
        self.value_node.data = value

    def __call__(self, run_info):
        return self.condition_func(run_info)

    def set_value(self, value):
        self.value = value

    def remove(self):
        if len(self.node.parent.children) == 1:
            self.node.parent.remove()
        self.node.remove()


class Compiler(NodeVisitor):
    def compile(self, tree, data_cls_getter=None, **kwargs):
        """Compile a raw AST into a form where conditional expressions
        are represented by ConditionalValue objects that can be evaluated
        at runtime.

        tree - The root node of the wptmanifest AST to compile

        data_cls_getter - A function taking two parameters; the previous
                          output node and the current ast node and returning
                          the class of the output node to use for the current
                          ast node
        """
        if data_cls_getter is None:
            self.data_cls_getter = lambda x, y: ManifestItem
        else:
            self.data_cls_getter = data_cls_getter

        self.tree = tree
        self.output_node = self._initial_output_node(tree, **kwargs)
        self.visit(tree)
        assert self.output_node is not None
        return self.output_node

    def compile_condition(self, condition):
        """Compile a ConditionalNode into a ConditionalValue.

        condition: A ConditionalNode"""
        data_node = DataNode()
        key_value_node = KeyValueNode()
        key_value_node.append(condition.copy())
        data_node.append(key_value_node)
        manifest_item = self.compile(data_node)
        return manifest_item._data[None][0]

    def _initial_output_node(self, node, **kwargs):
        return self.data_cls_getter(None, None)(node, **kwargs)

    def visit_DataNode(self, node):
        if node != self.tree:
            output_parent = self.output_node
            self.output_node = self.data_cls_getter(self.output_node, node)(node)
        else:
            output_parent = None

        assert self.output_node is not None

        for child in node.children:
            self.visit(child)

        if output_parent is not None:
            # Append to the parent *after* processing all the node data
            output_parent.append(self.output_node)
            self.output_node = self.output_node.parent

        assert self.output_node is not None

    def visit_KeyValueNode(self, node):
        key_values = []
        for child in node.children:
            condition, value = self.visit(child)
            key_values.append(ConditionalValue(child, condition))

        self.output_node._add_key_value(node, key_values)

    def visit_ValueNode(self, node):
        return (lambda x: True, node.data)

    def visit_ConditionalNode(self, node):
        return self.visit(node.children[0]), self.visit(node.children[1])

    def visit_StringNode(self, node):
        indexes = [self.visit(child) for child in node.children]

        def value(x):
            rv = node.data
            for index in indexes:
                rv = rv[index(x)]
            return rv
        return value

    def visit_NumberNode(self, node):
        if "." in node.data:
            return lambda x: float(node.data)
        else:
            return lambda x: int(node.data)

    def visit_VariableNode(self, node):
        indexes = [self.visit(child) for child in node.children]

        def value(x):
            data = x[node.data]
            for index in indexes:
                data = data[index(x)]
            return data
        return value

    def visit_IndexNode(self, node):
        assert len(node.children) == 1
        return self.visit(node.children[0])

    def visit_UnaryExpressionNode(self, node):
        assert len(node.children) == 2
        operator = self.visit(node.children[0])
        operand = self.visit(node.children[1])

        return lambda x: operator(operand(x))

    def visit_BinaryExpressionNode(self, node):
        assert len(node.children) == 3
        operator = self.visit(node.children[0])
        operand_0 = self.visit(node.children[1])
        operand_1 = self.visit(node.children[2])

        assert operand_0 is not None
        assert operand_1 is not None

        return lambda x: operator(operand_0(x), operand_1(x))

    def visit_UnaryOperatorNode(self, node):
        return {"not": operator.not_}[node.data]

    def visit_BinaryOperatorNode(self, node):
        return {"and": operator.and_,
                "or": operator.or_,
                "==": operator.eq,
                "!=": operator.ne}[node.data]


class ManifestItem(object):
    def __init__(self, node=None, **kwargs):
        self.node = node
        self.parent = None
        self.children = []
        self._data = {}

    def __repr__(self):
        return "<ManifestItem %s>" % (self.node.data)

    def __str__(self):
        rv = [repr(self)]
        for item in self.children:
            rv.extend("  %s" % line for line in str(item).split("\n"))
        return "\n".join(rv)

    @property
    def is_empty(self):
        if self._data:
            return False
        return all(child.is_empty for child in self.children)

    @property
    def root(self):
        node = self
        while node.parent is not None:
            node = node.parent
        return node

    @property
    def name(self):
        return self.node.data

    def has_key(self, key):
        for node in [self, self.root]:
            if key in node._data:
                return True
        return False

    def get(self, key, run_info=None):
        if run_info is None:
            run_info = {}

        for node in [self, self.root]:
            if key in node._data:
                for cond_value in node._data[key]:
                    try:
                        matches = cond_value(run_info)
                    except KeyError:
                        matches = False
                    if matches:
                        return cond_value.value
        raise KeyError

    def set(self, key, value, condition=None):
        # First try to update the existing value
        if key in self._data:
            cond_values = self._data[key]
            for cond_value in cond_values:
                if cond_value.condition_node == condition:
                    cond_value.value = value
                    return
            # If there isn't a conditional match reuse the existing KeyValueNode as the
            # parent
            node = None
            for child in self.node.children:
                if child.data == key:
                    node = child
                    break
            assert node is not None

        else:
            node = KeyValueNode(key)
            self.node.append(node)

        value_node = ValueNode(value)
        if condition is not None:
            conditional_node = ConditionalNode()
            conditional_node.append(condition)
            conditional_node.append(value_node)
            node.append(conditional_node)
            cond_value = Compiler().compile_condition(conditional_node)
        else:
            node.append(value_node)
            cond_value = ConditionalValue(value_node, lambda x: True)

        # Update the cache of child values. This is pretty annoying and maybe
        # it should just work directly on the tree
        if key not in self._data:
            self._data[key] = []
        if self._data[key] and self._data[key][-1].condition_node is None:
            self._data[key].insert(len(self._data[key]) - 1, cond_value)
        else:
            self._data[key].append(cond_value)

    def _add_key_value(self, node, values):
        """Called during construction to set a key-value node"""
        self._data[node.data] = values

    def append(self, child):
        self.children.append(child)
        child.parent = self
        if child.node.parent != self.node:
            self.node.append(child.node)
        return child

    def remove(self):
        if self.parent:
            self.parent._remove_child(self)

    def _remove_child(self, child):
        self.children.remove(child)
        child.parent = None

    def iterchildren(self, name=None):
        for item in self.children:
            if item.name == name or name is None:
                yield item

    def _flatten(self):
        rv = {}
        for node in [self, self.root]:
            for name, value in node._data.iteritems():
                if name not in rv:
                    rv[name] = value
        return rv

    def iteritems(self):
        for item in self._flatten().iteritems():
            yield item

    def iterkeys(self):
        for item in self._flatten().iterkeys():
            yield item

    def remove_value(self, key, value):
        self._data[key].remove(value)
        if not self._data[key]:
            del self._data[key]
        value.remove()


def compile_ast(ast, data_cls_getter=None, **kwargs):
    return Compiler().compile(ast, data_cls_getter=data_cls_getter, **kwargs)


def compile(stream, data_cls_getter=None, **kwargs):
    return compile_ast(parse(stream),
                       data_cls_getter=data_cls_getter,
                       **kwargs)
