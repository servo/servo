#default_value:foo
#include: other.manifest
#
#[test_name.js]
#  expected: ERROR
#
#  [subtest 1]
#    expected:
#      os == win: FAIL #This is a comment
#      PASS
#

# TODO: keep comments in the tree

from __future__ import unicode_literals

from six import binary_type, text_type, BytesIO, unichr
from six.moves import range

from .node import (Node, AtomNode, BinaryExpressionNode, BinaryOperatorNode,
                   ConditionalNode, DataNode, IndexNode, KeyValueNode, ListNode,
                   NumberNode, StringNode, UnaryExpressionNode,
                   UnaryOperatorNode, ValueNode, VariableNode)


class ParseError(Exception):
    def __init__(self, filename, line, detail):
        self.line = line
        self.filename = filename
        self.detail = detail
        self.message = "%s: %s line %s" % (self.detail, self.filename, self.line)
        Exception.__init__(self, self.message)

eol = object
group_start = object
group_end = object
digits = "0123456789"
open_parens = "[("
close_parens = "])"
parens = open_parens + close_parens
operator_chars = "=!"

unary_operators = ["not"]
binary_operators = ["==", "!=", "and", "or"]

operators = ["==", "!=", "not", "and", "or"]

atoms = {"True": True,
         "False": False,
         "Reset": object()}

def decode(s):
    assert isinstance(s, text_type)
    return s


def precedence(operator_node):
    return len(operators) - operators.index(operator_node.data)


class TokenTypes(object):
    def __init__(self):
        for type in ["group_start", "group_end", "paren", "list_start", "list_end", "separator", "ident", "string", "number", "atom", "eof"]:
            setattr(self, type, type)

token_types = TokenTypes()


class Tokenizer(object):
    def __init__(self):
        self.reset()

    def reset(self):
        self.indent_levels = [0]
        self.state = self.line_start_state
        self.next_state = self.data_line_state
        self.line_number = 0
        self.filename = ""

    def tokenize(self, stream):
        self.reset()
        assert not isinstance(stream, text_type)
        if isinstance(stream, bytes):
            stream = BytesIO(stream)
        if not hasattr(stream, "name"):
            self.filename = ""
        else:
            self.filename = stream.name

        self.next_line_state = self.line_start_state
        for i, line in enumerate(stream):
            assert isinstance(line, binary_type)
            self.state = self.next_line_state
            assert self.state is not None
            states = []
            self.next_line_state = None
            self.line_number = i + 1
            self.index = 0
            self.line = line.decode('utf-8').rstrip()
            assert isinstance(self.line, text_type)
            while self.state != self.eol_state:
                states.append(self.state)
                tokens = self.state()
                if tokens:
                    for token in tokens:
                        yield token
            self.state()
        while True:
            yield (token_types.eof, None)

    def char(self):
        if self.index == len(self.line):
            return eol
        return self.line[self.index]

    def consume(self):
        if self.index < len(self.line):
            self.index += 1

    def peek(self, length):
        return self.line[self.index:self.index + length]

    def skip_whitespace(self):
        while self.char() == " ":
            self.consume()

    def eol_state(self):
        if self.next_line_state is None:
            self.next_line_state = self.line_start_state

    def line_start_state(self):
        self.skip_whitespace()
        if self.char() == eol:
            self.state = self.eol_state
            return
        if self.index > self.indent_levels[-1]:
            self.indent_levels.append(self.index)
            yield (token_types.group_start, None)
        else:
            if self.index < self.indent_levels[-1]:
                while self.index < self.indent_levels[-1]:
                    self.indent_levels.pop()
                    yield (token_types.group_end, None)
                # This is terrible; if we were parsing an expression
                # then the next_state will be expr_or_value but when we deindent
                # it must always be a heading or key next so we go back to data_line_state
                self.next_state = self.data_line_state
            if self.index != self.indent_levels[-1]:
                raise ParseError(self.filename, self.line_number, "Unexpected indent")

        self.state = self.next_state

    def data_line_state(self):
        if self.char() == "[":
            yield (token_types.paren, self.char())
            self.consume()
            self.state = self.heading_state
        else:
            self.state = self.key_state

    def heading_state(self):
        rv = ""
        while True:
            c = self.char()
            if c == "\\":
                rv += self.consume_escape()
            elif c == "]":
                break
            elif c == eol:
                raise ParseError(self.filename, self.line_number, "EOL in heading")
            else:
                rv += c
                self.consume()

        yield (token_types.string, decode(rv))
        yield (token_types.paren, "]")
        self.consume()
        self.state = self.line_end_state
        self.next_state = self.data_line_state

    def key_state(self):
        rv = ""
        while True:
            c = self.char()
            if c == " ":
                self.skip_whitespace()
                if self.char() != ":":
                    raise ParseError(self.filename, self.line_number, "Space in key name")
                break
            elif c == ":":
                break
            elif c == eol:
                raise ParseError(self.filename, self.line_number, "EOL in key name (missing ':'?)")
            elif c == "\\":
                rv += self.consume_escape()
            else:
                rv += c
                self.consume()
        yield (token_types.string, decode(rv))
        yield (token_types.separator, ":")
        self.consume()
        self.state = self.after_key_state

    def after_key_state(self):
        self.skip_whitespace()
        c = self.char()
        if c == "#":
            self.next_state = self.expr_or_value_state
            self.state = self.comment_state
        elif c == eol:
            self.next_state = self.expr_or_value_state
            self.state = self.eol_state
        elif c == "[":
            self.state = self.list_start_state
        else:
            self.state = self.value_state

    def after_expr_state(self):
        self.skip_whitespace()
        c = self.char()
        if c == "#":
            self.next_state = self.after_expr_state
            self.state = self.comment_state
        elif c == eol:
            self.next_state = self.after_expr_state
            self.state = self.eol_state
        elif c == "[":
            self.state = self.list_start_state
        else:
            self.state = self.value_state

    def list_start_state(self):
        yield (token_types.list_start, "[")
        self.consume()
        self.state = self.list_value_start_state

    def list_value_start_state(self):
        self.skip_whitespace()
        if self.char() == "]":
            self.state = self.list_end_state
        elif self.char() in ("'", '"'):
            quote_char = self.char()
            self.consume()
            yield (token_types.string, self.consume_string(quote_char))
            self.skip_whitespace()
            if self.char() == "]":
                self.state = self.list_end_state
            elif self.char() != ",":
                raise ParseError(self.filename, self.line_number, "Junk after quoted string")
            self.consume()
        elif self.char() == "#":
            self.state = self.comment_state
            self.next_line_state = self.list_value_start_state
        elif self.char() == eol:
            self.next_line_state = self.list_value_start_state
            self.state = self.eol_state
        elif self.char() == ",":
            raise ParseError(self.filename, self.line_number, "List item started with separator")
        elif self.char() == "@":
            self.state = self.list_value_atom_state
        else:
            self.state = self.list_value_state

    def list_value_state(self):
        rv = ""
        spaces = 0
        while True:
            c = self.char()
            if c == "\\":
                escape = self.consume_escape()
                rv += escape
            elif c == eol:
                raise ParseError(self.filename, self.line_number, "EOL in list value")
            elif c == "#":
                raise ParseError(self.filename, self.line_number, "EOL in list value (comment)")
            elif c == ",":
                self.state = self.list_value_start_state
                self.consume()
                break
            elif c == " ":
                spaces += 1
                self.consume()
            elif c == "]":
                self.state = self.list_end_state
                self.consume()
                break
            else:
                rv += " " * spaces
                spaces = 0
                rv += c
                self.consume()

        if rv:
            yield (token_types.string, decode(rv))

    def list_value_atom_state(self):
        self.consume()
        for _, value in self.list_value_state():
            yield token_types.atom, value

    def list_end_state(self):
        self.consume()
        yield (token_types.list_end, "]")
        self.state = self.line_end_state

    def value_state(self):
        self.skip_whitespace()
        c = self.char()
        if c in ("'", '"'):
            quote_char = self.char()
            self.consume()
            yield (token_types.string, self.consume_string(quote_char))
            if self.char() == "#":
                self.state = self.comment_state
            else:
                self.state = self.line_end_state
        elif c == "@":
            self.consume()
            for _, value in self.value_inner_state():
                yield token_types.atom, value
        elif c == "[":
            self.state = self.list_start_state
        else:
            self.state = self.value_inner_state

    def value_inner_state(self):
        rv = ""
        spaces = 0
        while True:
            c = self.char()
            if c == "\\":
                rv += self.consume_escape()
            elif c == "#":
                self.state = self.comment_state
                break
            elif c == " ":
                # prevent whitespace before comments from being included in the value
                spaces += 1
                self.consume()
            elif c == eol:
                self.state = self.line_end_state
                break
            else:
                rv += " " * spaces
                spaces = 0
                rv += c
                self.consume()
        rv = decode(rv)
        if rv.startswith("if "):
            # Hack to avoid a problem where people write
            # disabled: if foo
            # and expect that to disable conditionally
            raise ParseError(self.filename, self.line_number, "Strings starting 'if ' must be quoted "
                             "(expressions must start on a newline and be indented)")
        yield (token_types.string, rv)

    def comment_state(self):
        while self.char() is not eol:
            self.consume()
        self.state = self.eol_state

    def line_end_state(self):
        self.skip_whitespace()
        c = self.char()
        if c == "#":
            self.state = self.comment_state
        elif c == eol:
            self.state = self.eol_state
        else:
            raise ParseError(self.filename, self.line_number, "Junk before EOL %s" % c)

    def consume_string(self, quote_char):
        rv = ""
        while True:
            c = self.char()
            if c == "\\":
                rv += self.consume_escape()
            elif c == quote_char:
                self.consume()
                break
            elif c == eol:
                raise ParseError(self.filename, self.line_number, "EOL in quoted string")
            else:
                rv += c
                self.consume()

        return decode(rv)

    def expr_or_value_state(self):
        if self.peek(3) == "if ":
            self.state = self.expr_state
        else:
            self.state = self.value_state

    def expr_state(self):
        self.skip_whitespace()
        c = self.char()
        if c == eol:
            raise ParseError(self.filename, self.line_number, "EOL in expression")
        elif c in "'\"":
            self.consume()
            yield (token_types.string, self.consume_string(c))
        elif c == "#":
            raise ParseError(self.filename, self.line_number, "Comment before end of expression")
        elif c == ":":
            yield (token_types.separator, c)
            self.consume()
            self.state = self.after_expr_state
        elif c in parens:
            self.consume()
            yield (token_types.paren, c)
        elif c in ("!", "="):
            self.state = self.operator_state
        elif c in digits:
            self.state = self.digit_state
        else:
            self.state = self.ident_state

    def operator_state(self):
        # Only symbolic operators
        index_0 = self.index
        while True:
            c = self.char()
            if c == eol:
                break
            elif c in operator_chars:
                self.consume()
            else:
                self.state = self.expr_state
                break
        yield (token_types.ident, self.line[index_0:self.index])

    def digit_state(self):
        index_0 = self.index
        seen_dot = False
        while True:
            c = self.char()
            if c == eol:
                break
            elif c in digits:
                self.consume()
            elif c == ".":
                if seen_dot:
                    raise ParseError(self.filename, self.line_number, "Invalid number")
                self.consume()
                seen_dot = True
            elif c in parens:
                break
            elif c in operator_chars:
                break
            elif c == " ":
                break
            elif c == ":":
                break
            else:
                raise ParseError(self.filename, self.line_number, "Invalid character in number")

        self.state = self.expr_state
        yield (token_types.number, self.line[index_0:self.index])

    def ident_state(self):
        index_0 = self.index
        while True:
            c = self.char()
            if c == eol:
                break
            elif c == ".":
                break
            elif c in parens:
                break
            elif c in operator_chars:
                break
            elif c == " ":
                break
            elif c == ":":
                break
            else:
                self.consume()
        self.state = self.expr_state
        yield (token_types.ident, self.line[index_0:self.index])

    def consume_escape(self):
        assert self.char() == "\\"
        self.consume()
        c = self.char()
        self.consume()
        if c == "x":
            return self.decode_escape(2)
        elif c == "u":
            return self.decode_escape(4)
        elif c == "U":
            return self.decode_escape(6)
        elif c in ["a", "b", "f", "n", "r", "t", "v"]:
            return eval(r"'\%s'" % c)
        elif c is eol:
            raise ParseError(self.filename, self.line_number, "EOL in escape")
        else:
            return c

    def decode_escape(self, length):
        value = 0
        for i in range(length):
            c = self.char()
            value *= 16
            value += self.escape_value(c)
            self.consume()

        return unichr(value)

    def escape_value(self, c):
        if '0' <= c <= '9':
            return ord(c) - ord('0')
        elif 'a' <= c <= 'f':
            return ord(c) - ord('a') + 10
        elif 'A' <= c <= 'F':
            return ord(c) - ord('A') + 10
        else:
            raise ParseError(self.filename, self.line_number, "Invalid character escape")


class Parser(object):
    def __init__(self):
        self.reset()

    def reset(self):
        self.token = None
        self.unary_operators = "!"
        self.binary_operators = frozenset(["&&", "||", "=="])
        self.tokenizer = Tokenizer()
        self.token_generator = None
        self.tree = Treebuilder(DataNode(None))
        self.expr_builder = None
        self.expr_builders = []

    def parse(self, input):
        try:
            self.reset()
            self.token_generator = self.tokenizer.tokenize(input)
            self.consume()
            self.manifest()
            return self.tree.node
        except Exception as e:
            if not isinstance(e, ParseError):
                raise ParseError(self.tokenizer.filename,
                                 self.tokenizer.line_number,
                                 str(e))
            raise

    def consume(self):
        self.token = next(self.token_generator)

    def expect(self, type, value=None):
        if self.token[0] != type:
            raise ParseError(self.tokenizer.filename, self.tokenizer.line_number,
                             "Token '{}' doesn't equal expected type '{}'".format(self.token[0], type))
        if value is not None:
            if self.token[1] != value:
                raise ParseError(self.tokenizer.filename, self.tokenizer.line_number,
                                 "Token '{}' doesn't equal expected value '{}'".format(self.token[1], value))

        self.consume()

    def manifest(self):
        self.data_block()
        self.expect(token_types.eof)

    def data_block(self):
        while self.token[0] == token_types.string:
            self.tree.append(KeyValueNode(self.token[1]))
            self.consume()
            self.expect(token_types.separator)
            self.value_block()
            self.tree.pop()

        while self.token == (token_types.paren, "["):
            self.consume()
            if self.token[0] != token_types.string:
                raise ParseError(self.tokenizer.filename, self.tokenizer.line_number,
                                 "Token '{}' is not a string".format(self.token[0]))
            self.tree.append(DataNode(self.token[1]))
            self.consume()
            self.expect(token_types.paren, "]")
            if self.token[0] == token_types.group_start:
                self.consume()
                self.data_block()
                self.eof_or_end_group()
            self.tree.pop()

    def eof_or_end_group(self):
        if self.token[0] != token_types.eof:
            self.expect(token_types.group_end)

    def value_block(self):
        if self.token[0] == token_types.list_start:
            self.consume()
            self.list_value()
        elif self.token[0] == token_types.string:
            self.value()
        elif self.token[0] == token_types.group_start:
            self.consume()
            self.expression_values()
            if self.token[0] == token_types.string:
                self.value()
            elif self.token[0] == token_types.list_start:
                self.consume()
                self.list_value()
            self.eof_or_end_group()
        elif self.token[0] == token_types.atom:
            self.atom()
        else:
            raise ParseError(self.tokenizer.filename, self.tokenizer.line_number,
                             "Token '{}' is not a known type".format(self.token[0]))

    def list_value(self):
        self.tree.append(ListNode())
        while self.token[0] in (token_types.atom, token_types.string):
            if self.token[0] == token_types.atom:
                self.atom()
            else:
                self.value()
        self.expect(token_types.list_end)
        self.tree.pop()

    def expression_values(self):
        while self.token == (token_types.ident, "if"):
            self.consume()
            self.tree.append(ConditionalNode())
            self.expr_start()
            self.expect(token_types.separator)
            self.value_block()
            self.tree.pop()

    def value(self):
        self.tree.append(ValueNode(self.token[1]))
        self.consume()
        self.tree.pop()

    def atom(self):
        if self.token[1] not in atoms:
            raise ParseError(self.tokenizer.filename, self.tokenizer.line_number, "Unrecognised symbol @%s" % self.token[1])
        self.tree.append(AtomNode(atoms[self.token[1]]))
        self.consume()
        self.tree.pop()

    def expr_start(self):
        self.expr_builder = ExpressionBuilder(self.tokenizer)
        self.expr_builders.append(self.expr_builder)
        self.expr()
        expression = self.expr_builder.finish()
        self.expr_builders.pop()
        self.expr_builder = self.expr_builders[-1] if self.expr_builders else None
        if self.expr_builder:
            self.expr_builder.operands[-1].children[-1].append(expression)
        else:
            self.tree.append(expression)
            self.tree.pop()

    def expr(self):
        self.expr_operand()
        while (self.token[0] == token_types.ident and self.token[1] in binary_operators):
            self.expr_bin_op()
            self.expr_operand()

    def expr_operand(self):
        if self.token == (token_types.paren, "("):
            self.consume()
            self.expr_builder.left_paren()
            self.expr()
            self.expect(token_types.paren, ")")
            self.expr_builder.right_paren()
        elif self.token[0] == token_types.ident and self.token[1] in unary_operators:
            self.expr_unary_op()
            self.expr_operand()
        elif self.token[0] in [token_types.string, token_types.ident]:
            self.expr_value()
        elif self.token[0] == token_types.number:
            self.expr_number()
        else:
            raise ParseError(self.tokenizer.filename, self.tokenizer.line_number, "Unrecognised operand")

    def expr_unary_op(self):
        if self.token[1] in unary_operators:
            self.expr_builder.push_operator(UnaryOperatorNode(self.token[1]))
            self.consume()
        else:
            raise ParseError(self.tokenizer.filename, self.tokenizer.line_number, "Expected unary operator")

    def expr_bin_op(self):
        if self.token[1] in binary_operators:
            self.expr_builder.push_operator(BinaryOperatorNode(self.token[1]))
            self.consume()
        else:
            raise ParseError(self.tokenizer.filename, self.tokenizer.line_number, "Expected binary operator")

    def expr_value(self):
        node_type = {token_types.string: StringNode,
                     token_types.ident: VariableNode}[self.token[0]]
        self.expr_builder.push_operand(node_type(self.token[1]))
        self.consume()
        if self.token == (token_types.paren, "["):
            self.consume()
            self.expr_builder.operands[-1].append(IndexNode())
            self.expr_start()
            self.expect(token_types.paren, "]")

    def expr_number(self):
        self.expr_builder.push_operand(NumberNode(self.token[1]))
        self.consume()


class Treebuilder(object):
    def __init__(self, root):
        self.root = root
        self.node = root

    def append(self, node):
        assert isinstance(node, Node)
        self.node.append(node)
        self.node = node
        assert self.node is not None
        return node

    def pop(self):
        node = self.node
        self.node = self.node.parent
        assert self.node is not None
        return node


class ExpressionBuilder(object):
    def __init__(self, tokenizer):
        self.operands = []
        self.operators = [None]
        self.tokenizer = tokenizer

    def finish(self):
        while self.operators[-1] is not None:
            self.pop_operator()
        rv = self.pop_operand()
        assert self.is_empty()
        return rv

    def left_paren(self):
        self.operators.append(None)

    def right_paren(self):
        while self.operators[-1] is not None:
            self.pop_operator()
            if not self.operators:
                raise ParseError(self.tokenizer.filename, self.tokenizer.line,
                                 "Unbalanced parens")

        assert self.operators.pop() is None

    def push_operator(self, operator):
        assert operator is not None
        while self.precedence(self.operators[-1]) > self.precedence(operator):
            self.pop_operator()

        self.operators.append(operator)

    def pop_operator(self):
        operator = self.operators.pop()
        if isinstance(operator, BinaryOperatorNode):
            operand_1 = self.operands.pop()
            operand_0 = self.operands.pop()
            self.operands.append(BinaryExpressionNode(operator, operand_0, operand_1))
        else:
            operand_0 = self.operands.pop()
            self.operands.append(UnaryExpressionNode(operator, operand_0))

    def push_operand(self, node):
        self.operands.append(node)

    def pop_operand(self):
        return self.operands.pop()

    def is_empty(self):
        return len(self.operands) == 0 and all(item is None for item in self.operators)

    def precedence(self, operator):
        if operator is None:
            return 0
        return precedence(operator)


def parse(stream):
    p = Parser()
    return p.parse(stream)
