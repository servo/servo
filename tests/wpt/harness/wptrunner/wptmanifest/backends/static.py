# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import operator

from ..node import NodeVisitor
from ..parser import parse


class Compiler(NodeVisitor):
    """Compiler backend that evaluates conditional expressions
    to give static output"""

    def compile(self, tree, expr_data, data_cls_getter=None, **kwargs):
        """Compile a raw AST into a form with conditional expressions
        evaluated.

        tree - The root node of the wptmanifest AST to compile

        expr_data - A dictionary of key / value pairs to use when
                    evaluating conditional expressions

        data_cls_getter - A function taking two parameters; the previous
                          output node and the current ast node and returning
                          the class of the output node to use for the current
                          ast node
        """

        self._kwargs = kwargs
        self.expr_data = expr_data

        if data_cls_getter is None:
            self.data_cls_getter = lambda x, y: ManifestItem
        else:
            self.data_cls_getter = data_cls_getter

        self.output_node = None
        self.visit(tree)
        return self.output_node

    def visit_DataNode(self, node):
        output_parent = self.output_node
        if self.output_node is None:
            assert node.parent is None
            self.output_node = self.data_cls_getter(None, None)(None, **self._kwargs)
        else:
            self.output_node = self.data_cls_getter(self.output_node, node)(node.data)

        for child in node.children:
            self.visit(child)

        if output_parent is not None:
            output_parent.append(self.output_node)
            self.output_node = self.output_node.parent

    def visit_KeyValueNode(self, node):
        key_name = node.data
        key_value = None
        for child in node.children:
            value = self.visit(child)
            if value is not None:
                key_value = value
                break
        if key_value is not None:
            self.output_node.set(key_name, key_value)

    def visit_ValueNode(self, node):
        return node.data

    def visit_AtomNode(self, node):
        return node.data

    def visit_ListNode(self, node):
        return [self.visit(child) for child in node.children]

    def visit_ConditionalNode(self, node):
        assert len(node.children) == 2
        if self.visit(node.children[0]):
            return self.visit(node.children[1])

    def visit_StringNode(self, node):
        value = node.data
        for child in node.children:
            value = self.visit(child)(value)
        return value

    def visit_NumberNode(self, node):
        if "." in node.data:
            return float(node.data)
        else:
            return int(node.data)

    def visit_VariableNode(self, node):
        value = self.expr_data[node.data]
        for child in node.children:
            value = self.visit(child)(value)
        return value

    def visit_IndexNode(self, node):
        assert len(node.children) == 1
        index = self.visit(node.children[0])
        return lambda x: x[index]

    def visit_UnaryExpressionNode(self, node):
        assert len(node.children) == 2
        operator = self.visit(node.children[0])
        operand = self.visit(node.children[1])

        return operator(operand)

    def visit_BinaryExpressionNode(self, node):
        assert len(node.children) == 3
        operator = self.visit(node.children[0])
        operand_0 = self.visit(node.children[1])
        operand_1 = self.visit(node.children[2])

        return operator(operand_0, operand_1)

    def visit_UnaryOperatorNode(self, node):
        return {"not": operator.not_}[node.data]

    def visit_BinaryOperatorNode(self, node):
        return {"and": operator.and_,
                "or": operator.or_,
                "==": operator.eq,
                "!=": operator.ne}[node.data]


class ManifestItem(object):
    def __init__(self, name, **kwargs):
        self.parent = None
        self.name = name
        self.children = []
        self._data = {}

    def __repr__(self):
        return "<ManifestItem %s>" % (self.name)

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

    def has_key(self, key):
        for node in [self, self.root]:
            if key in node._data:
                return True
        return False

    def get(self, key):
        for node in [self, self.root]:
            if key in node._data:
                return node._data[key]
        raise KeyError

    def set(self, name, value):
        self._data[name] = value

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

    def itervalues(self):
        for item in self._flatten().itervalues():
            yield item

    def append(self, child):
        child.parent = self
        self.children.append(child)
        return child


def compile_ast(ast, expr_data, data_cls_getter=None, **kwargs):
    return Compiler().compile(ast,
                              expr_data,
                              data_cls_getter=data_cls_getter,
                              **kwargs)


def compile(stream, expr_data, data_cls_getter=None, **kwargs):
    return compile_ast(parse(stream),
                       expr_data,
                       data_cls_getter=data_cls_getter,
                       **kwargs)
