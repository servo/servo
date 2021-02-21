class NodeVisitor(object):
    def visit(self, node):
        # This is ugly as hell, but we don't have multimethods and
        # they aren't trivial to fake without access to the class
        # object from the class body
        func = getattr(self, "visit_%s" % (node.__class__.__name__))
        return func(node)


class Node(object):
    def __init__(self, data=None):
        self.data = data
        self.parent = None
        self.children = []

    def append(self, other):
        other.parent = self
        self.children.append(other)

    def remove(self):
        self.parent.children.remove(self)

    def __repr__(self):
        return "<%s %s>" % (self.__class__.__name__, self.data)

    def __str__(self):
        rv = [repr(self)]
        for item in self.children:
            rv.extend("  %s" % line for line in str(item).split("\n"))
        return "\n".join(rv)

    def __eq__(self, other):
        if not (self.__class__ == other.__class__ and
                self.data == other.data and
                len(self.children) == len(other.children)):
            return False
        for child, other_child in zip(self.children, other.children):
            if not child == other_child:
                return False
        return True

    def copy(self):
        new = self.__class__(self.data)
        for item in self.children:
            new.append(item.copy())
        return new


class DataNode(Node):
    def append(self, other):
        # Append that retains the invariant that child data nodes
        # come after child nodes of other types
        other.parent = self
        if isinstance(other, DataNode):
            self.children.append(other)
        else:
            index = len(self.children)
            while index > 0 and isinstance(self.children[index - 1], DataNode):
                index -= 1
            for i in range(index):
                if other.data == self.children[i].data:
                    raise ValueError("Duplicate key %s" % self.children[i].data)
            self.children.insert(index, other)


class KeyValueNode(Node):
    def append(self, other):
        # Append that retains the invariant that conditional nodes
        # come before unconditional nodes
        other.parent = self
        if not isinstance(other, (ListNode, ValueNode, ConditionalNode)):
            raise TypeError
        if isinstance(other, (ListNode, ValueNode)):
            if self.children:
                assert not isinstance(self.children[-1], (ListNode, ValueNode))
            self.children.append(other)
        else:
            if self.children and isinstance(self.children[-1], ValueNode):
                self.children.insert(len(self.children) - 1, other)
            else:
                self.children.append(other)


class ListNode(Node):
    def append(self, other):
        other.parent = self
        self.children.append(other)


class ValueNode(Node):
    def append(self, other):
        raise TypeError


class AtomNode(ValueNode):
    pass


class ConditionalNode(Node):
    def append(self, other):
        if not len(self.children):
            if not isinstance(other, (BinaryExpressionNode, UnaryExpressionNode, VariableNode)):
                raise TypeError
        else:
            if len(self.children) > 1:
                raise ValueError
            if not isinstance(other, (ListNode, ValueNode)):
                raise TypeError
        other.parent = self
        self.children.append(other)


class UnaryExpressionNode(Node):
    def __init__(self, operator, operand):
        Node.__init__(self)
        self.append(operator)
        self.append(operand)

    def append(self, other):
        Node.append(self, other)
        assert len(self.children) <= 2

    def copy(self):
        new = self.__class__(self.children[0].copy(),
                             self.children[1].copy())
        return new


class BinaryExpressionNode(Node):
    def __init__(self, operator, operand_0, operand_1):
        Node.__init__(self)
        self.append(operator)
        self.append(operand_0)
        self.append(operand_1)

    def append(self, other):
        Node.append(self, other)
        assert len(self.children) <= 3

    def copy(self):
        new = self.__class__(self.children[0].copy(),
                             self.children[1].copy(),
                             self.children[2].copy())
        return new


class UnaryOperatorNode(Node):
    def append(self, other):
        raise TypeError


class BinaryOperatorNode(Node):
    def append(self, other):
        raise TypeError


class IndexNode(Node):
    pass


class VariableNode(Node):
    pass


class StringNode(Node):
    pass


class NumberNode(ValueNode):
    pass
