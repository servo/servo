import abc
from six import iteritems, iterkeys, itervalues

from ..node import NodeVisitor
from ..parser import parse


class Compiler(NodeVisitor):
    __metaclass__ = abc.ABCMeta

    def compile(self, tree, data_cls_getter=None, **kwargs):
        self._kwargs = kwargs
        return self._compile(tree, data_cls_getter, **kwargs)

    def _compile(self, tree, data_cls_getter=None, **kwargs):
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
        if hasattr(self.output_node, "set_defaults"):
            self.output_node.set_defaults()
        assert self.output_node is not None
        return self.output_node

    def _initial_output_node(self, node, **kwargs):
        return self.data_cls_getter(None, None)(node, **kwargs)

    def visit_DataNode(self, node):
        if node != self.tree:
            output_parent = self.output_node
            self.output_node = self.data_cls_getter(self.output_node, node)(node, **self._kwargs)
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

    @abc.abstractmethod
    def visit_KeyValueNode(self, node):
        pass

    def visit_ListNode(self, node):
        return [self.visit(child) for child in node.children]

    def visit_ValueNode(self, node):
        return node.data

    def visit_AtomNode(self, node):
        return node.data

    @abc.abstractmethod
    def visit_ConditionalNode(self, node):
        pass

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
            return float(node.data)
        else:
            return int(node.data)

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

    @abc.abstractmethod
    def visit_UnaryExpressionNode(self, node):
        pass

    @abc.abstractmethod
    def visit_BinaryExpressionNode(self, node):
        pass

    @abc.abstractmethod
    def visit_UnaryOperatorNode(self, node):
        pass

    @abc.abstractmethod
    def visit_BinaryOperatorNode(self, node):
        pass


class ManifestItem(object):
    def __init__(self, node, **kwargs):
        self.parent = None
        self.node = node
        self.children = []
        self._data = {}

    def __repr__(self):
        return "<%s %s>" % (self.__class__, self.node.data)

    def __str__(self):
        rv = [repr(self)]
        for item in self.children:
            rv.extend("  %s" % line for line in str(item).split("\n"))
        return "\n".join(rv)

    def set_defaults(self):
        pass

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

    def get(self, key):
        for node in [self, self.root]:
            if key in node._data:
                return node._data[key]
        raise KeyError

    def set(self, name, value):
        self._data[name] = value

    def remove(self):
        if self.parent:
            self.parent.children.remove(self)
            self.parent = None

    def iterchildren(self, name=None):
        for item in self.children:
            if item.name == name or name is None:
                yield item

    def has_key(self, key):
        for node in [self, self.root]:
            if key in node._data:
                return True
        return False

    def _flatten(self):
        rv = {}
        for node in [self, self.root]:
            for name, value in iteritems(node._data):
                if name not in rv:
                    rv[name] = value
        return rv

    def iteritems(self):
        for item in iteritems(self._flatten()):
            yield item

    def iterkeys(self):
        for item in iterkeys(self._flatten()):
            yield item

    def itervalues(self):
        for item in itervalues(self._flatten()):
            yield item

    def append(self, child):
        child.parent = self
        self.children.append(child)
        return child


def compile_ast(compiler, ast, data_cls_getter=None, **kwargs):
    return compiler().compile(ast,
                              data_cls_getter=data_cls_getter,
                              **kwargs)


def compile(compiler, stream, data_cls_getter=None, **kwargs):
    return compile_ast(compiler,
                       parse(stream),
                       data_cls_getter=data_cls_getter,
                       **kwargs)
