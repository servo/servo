import operator

from . import base
from ..parser import parse


class Compiler(base.Compiler):
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

        return self._compile(tree, data_cls_getter, **kwargs)

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

    def visit_ConditionalNode(self, node):
        assert len(node.children) == 2
        if self.visit(node.children[0]):
            return self.visit(node.children[1])

    def visit_StringNode(self, node):
        value = node.data
        for child in node.children:
            value = self.visit(child)(value)
        return value

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
