# mypy: allow-untyped-defs

from six import ensure_text

from .node import NodeVisitor, ValueNode, ListNode, BinaryExpressionNode
from .parser import atoms, precedence, token_types

atom_names = {v: "@%s" % k for (k,v) in atoms.items()}

named_escapes = {"\a", "\b", "\f", "\n", "\r", "\t", "\v"}

def escape(string, extras=""):
    # Assumes input bytes are either UTF8 bytes or unicode.
    rv = ""
    for c in string:
        if c in named_escapes:
            rv += c.encode("unicode_escape").decode()
        elif c == "\\":
            rv += "\\\\"
        elif c < '\x20':
            rv += "\\x%02x" % ord(c)
        elif c in extras:
            rv += "\\" + c
        else:
            rv += c
    return ensure_text(rv)


class ManifestSerializer(NodeVisitor):
    def __init__(self, skip_empty_data=False):
        self.skip_empty_data = skip_empty_data

    def serialize(self, root):
        self.indent = 2
        rv = "\n".join(self.visit(root))
        if not rv:
            return rv
        rv = rv.strip()
        if rv[-1] != "\n":
            rv = rv + "\n"
        return rv

    def visit(self, node):
        lines = super().visit(node)
        comments = [f"#{comment}" for _, comment in node.comments]
        # Simply checking if the first line contains '#' is less than ideal; the
        # character might be escaped or within a string.
        if lines and "#" not in lines[0]:
            for i, (token_type, comment) in enumerate(node.comments):
                if token_type == token_types.inline_comment:
                    lines[0] += f"  #{comment}"
                    comments.pop(i)
                    break
        return comments + lines

    def visit_DataNode(self, node):
        rv = []
        if not self.skip_empty_data or node.children:
            if node.data:
                rv.append("[%s]" % escape(node.data, extras="]"))
                indent = self.indent * " "
            else:
                indent = ""

            for child in node.children:
                rv.extend("%s%s" % (indent if item else "", item) for item in self.visit(child))

            if node.parent:
                rv.append("")

        return rv

    def visit_KeyValueNode(self, node):
        rv = [escape(node.data, ":") + ":"]
        indent = " " * self.indent

        if len(node.children) == 1 and isinstance(node.children[0], (ValueNode, ListNode)):
            rv[0] += " %s" % self.visit(node.children[0])[0]
        else:
            for child in node.children:
                rv.extend(indent + line for line in self.visit(child))

        return rv

    def visit_ListNode(self, node):
        rv = ["["]
        rv.extend(", ".join(self.visit(child)[0] for child in node.children))
        rv.append("]")
        return ["".join(rv)]

    def visit_ValueNode(self, node):
        data = ensure_text(node.data)
        if ("#" in data or
            data.startswith("if ") or
            (isinstance(node.parent, ListNode) and
             ("," in data or "]" in data))):
            if "\"" in data:
                quote = "'"
            else:
                quote = "\""
        else:
            quote = ""
        return [quote + escape(data, extras=quote) + quote]

    def visit_AtomNode(self, node):
        return [atom_names[node.data]]

    def visit_ConditionalNode(self, node):
        return ["if %s: %s" % tuple(self.visit(item)[0] for item in node.children)]

    def visit_StringNode(self, node):
        rv = ["\"%s\"" % escape(node.data, extras="\"")]
        for child in node.children:
            rv[0] += self.visit(child)[0]
        return rv

    def visit_NumberNode(self, node):
        return [ensure_text(node.data)]

    def visit_VariableNode(self, node):
        rv = escape(node.data)
        for child in node.children:
            rv += self.visit(child)
        return [rv]

    def visit_IndexNode(self, node):
        assert len(node.children) == 1
        return ["[%s]" % self.visit(node.children[0])[0]]

    def visit_UnaryExpressionNode(self, node):
        children = []
        for child in node.children:
            child_str = self.visit(child)[0]
            if isinstance(child, BinaryExpressionNode):
                child_str = "(%s)" % child_str
            children.append(child_str)
        return [" ".join(children)]

    def visit_BinaryExpressionNode(self, node):
        assert len(node.children) == 3
        children = []
        for child_index in [1, 0, 2]:
            child = node.children[child_index]
            child_str = self.visit(child)[0]
            if (isinstance(child, BinaryExpressionNode) and
                precedence(node.children[0]) < precedence(child.children[0])):
                child_str = "(%s)" % child_str
            children.append(child_str)
        return [" ".join(children)]

    def visit_UnaryOperatorNode(self, node):
        return [ensure_text(node.data)]

    def visit_BinaryOperatorNode(self, node):
        return [ensure_text(node.data)]


def serialize(tree, *args, **kwargs):
    s = ManifestSerializer(*args, **kwargs)
    return s.serialize(tree)
