# GardenSnake - a parser generator demonstration program
#
# This implements a modified version of a subset of Python:
#  - only 'def', 'return' and 'if' statements
#  - 'if' only has 'then' clause (no elif nor else)
#  - single-quoted strings only, content in raw format
#  - numbers are decimal.Decimal instances (not integers or floats)
#  - no print statment; use the built-in 'print' function
#  - only < > == + - / * implemented (and unary + -)
#  - assignment and tuple assignment work
#  - no generators of any sort
#  - no ... well, no quite a lot

# Why?  I'm thinking about a new indentation-based configuration
# language for a project and wanted to figure out how to do it.  Once
# I got that working I needed a way to test it out.  My original AST
# was dumb so I decided to target Python's AST and compile it into
# Python code.  Plus, it's pretty cool that it only took a day or so
# from sitting down with Ply to having working code.

# This uses David Beazley's Ply from http://www.dabeaz.com/ply/

# This work is hereby released into the Public Domain. To view a copy of
# the public domain dedication, visit
# http://creativecommons.org/licenses/publicdomain/ or send a letter to
# Creative Commons, 543 Howard Street, 5th Floor, San Francisco,
# California, 94105, USA.
#
# Portions of this work are derived from Python's Grammar definition
# and may be covered under the Python copyright and license
#
#          Andrew Dalke / Dalke Scientific Software, LLC
#             30 August 2006 / Cape Town, South Africa

# Changelog:
#  30 August - added link to CC license; removed the "swapcase" encoding

# Modifications for inclusion in PLY distribution
import sys
sys.path.insert(0, "../..")
from ply import *

##### Lexer ######
#import lex
import decimal

tokens = (
    'DEF',
    'IF',
    'NAME',
    'NUMBER',  # Python decimals
    'STRING',  # single quoted strings only; syntax of raw strings
    'LPAR',
    'RPAR',
    'COLON',
    'EQ',
    'ASSIGN',
    'LT',
    'GT',
    'PLUS',
    'MINUS',
    'MULT',
    'DIV',
    'RETURN',
    'WS',
    'NEWLINE',
    'COMMA',
    'SEMICOLON',
    'INDENT',
    'DEDENT',
    'ENDMARKER',
)

#t_NUMBER = r'\d+'
# taken from decmial.py but without the leading sign


def t_NUMBER(t):
    r"""(\d+(\.\d*)?|\.\d+)([eE][-+]? \d+)?"""
    t.value = decimal.Decimal(t.value)
    return t


def t_STRING(t):
    r"'([^\\']+|\\'|\\\\)*'"  # I think this is right ...
    t.value = t.value[1:-1].decode("string-escape")  # .swapcase() # for fun
    return t

t_COLON = r':'
t_EQ = r'=='
t_ASSIGN = r'='
t_LT = r'<'
t_GT = r'>'
t_PLUS = r'\+'
t_MINUS = r'-'
t_MULT = r'\*'
t_DIV = r'/'
t_COMMA = r','
t_SEMICOLON = r';'

# Ply nicely documented how to do this.

RESERVED = {
    "def": "DEF",
    "if": "IF",
    "return": "RETURN",
}


def t_NAME(t):
    r'[a-zA-Z_][a-zA-Z0-9_]*'
    t.type = RESERVED.get(t.value, "NAME")
    return t

# Putting this before t_WS let it consume lines with only comments in
# them so the latter code never sees the WS part.  Not consuming the
# newline.  Needed for "if 1: #comment"


def t_comment(t):
    r"[ ]*\043[^\n]*"  # \043 is '#'
    pass


# Whitespace
def t_WS(t):
    r' [ ]+ '
    if t.lexer.at_line_start and t.lexer.paren_count == 0:
        return t

# Don't generate newline tokens when inside of parenthesis, eg
#   a = (1,
#        2, 3)


def t_newline(t):
    r'\n+'
    t.lexer.lineno += len(t.value)
    t.type = "NEWLINE"
    if t.lexer.paren_count == 0:
        return t


def t_LPAR(t):
    r'\('
    t.lexer.paren_count += 1
    return t


def t_RPAR(t):
    r'\)'
    # check for underflow?  should be the job of the parser
    t.lexer.paren_count -= 1
    return t


def t_error(t):
    raise SyntaxError("Unknown symbol %r" % (t.value[0],))
    print "Skipping", repr(t.value[0])
    t.lexer.skip(1)

# I implemented INDENT / DEDENT generation as a post-processing filter

# The original lex token stream contains WS and NEWLINE characters.
# WS will only occur before any other tokens on a line.

# I have three filters.  One tags tokens by adding two attributes.
# "must_indent" is True if the token must be indented from the
# previous code.  The other is "at_line_start" which is True for WS
# and the first non-WS/non-NEWLINE on a line.  It flags the check so
# see if the new line has changed indication level.

# Python's syntax has three INDENT states
#  0) no colon hence no need to indent
#  1) "if 1: go()" - simple statements have a COLON but no need for an indent
#  2) "if 1:\n  go()" - complex statements have a COLON NEWLINE and must indent
NO_INDENT = 0
MAY_INDENT = 1
MUST_INDENT = 2

# only care about whitespace at the start of a line


def track_tokens_filter(lexer, tokens):
    lexer.at_line_start = at_line_start = True
    indent = NO_INDENT
    saw_colon = False
    for token in tokens:
        token.at_line_start = at_line_start

        if token.type == "COLON":
            at_line_start = False
            indent = MAY_INDENT
            token.must_indent = False

        elif token.type == "NEWLINE":
            at_line_start = True
            if indent == MAY_INDENT:
                indent = MUST_INDENT
            token.must_indent = False

        elif token.type == "WS":
            assert token.at_line_start == True
            at_line_start = True
            token.must_indent = False

        else:
            # A real token; only indent after COLON NEWLINE
            if indent == MUST_INDENT:
                token.must_indent = True
            else:
                token.must_indent = False
            at_line_start = False
            indent = NO_INDENT

        yield token
        lexer.at_line_start = at_line_start


def _new_token(type, lineno):
    tok = lex.LexToken()
    tok.type = type
    tok.value = None
    tok.lineno = lineno
    return tok

# Synthesize a DEDENT tag


def DEDENT(lineno):
    return _new_token("DEDENT", lineno)

# Synthesize an INDENT tag


def INDENT(lineno):
    return _new_token("INDENT", lineno)


# Track the indentation level and emit the right INDENT / DEDENT events.
def indentation_filter(tokens):
    # A stack of indentation levels; will never pop item 0
    levels = [0]
    token = None
    depth = 0
    prev_was_ws = False
    for token in tokens:
        # if 1:
        # print "Process", token,
        # if token.at_line_start:
        # print "at_line_start",
        # if token.must_indent:
        # print "must_indent",
        # print

        # WS only occurs at the start of the line
        # There may be WS followed by NEWLINE so
        # only track the depth here.  Don't indent/dedent
        # until there's something real.
        if token.type == "WS":
            assert depth == 0
            depth = len(token.value)
            prev_was_ws = True
            # WS tokens are never passed to the parser
            continue

        if token.type == "NEWLINE":
            depth = 0
            if prev_was_ws or token.at_line_start:
                # ignore blank lines
                continue
            # pass the other cases on through
            yield token
            continue

        # then it must be a real token (not WS, not NEWLINE)
        # which can affect the indentation level

        prev_was_ws = False
        if token.must_indent:
            # The current depth must be larger than the previous level
            if not (depth > levels[-1]):
                raise IndentationError("expected an indented block")

            levels.append(depth)
            yield INDENT(token.lineno)

        elif token.at_line_start:
            # Must be on the same level or one of the previous levels
            if depth == levels[-1]:
                # At the same level
                pass
            elif depth > levels[-1]:
                raise IndentationError(
                    "indentation increase but not in new block")
            else:
                # Back up; but only if it matches a previous level
                try:
                    i = levels.index(depth)
                except ValueError:
                    raise IndentationError("inconsistent indentation")
                for _ in range(i + 1, len(levels)):
                    yield DEDENT(token.lineno)
                    levels.pop()

        yield token

    ### Finished processing ###

    # Must dedent any remaining levels
    if len(levels) > 1:
        assert token is not None
        for _ in range(1, len(levels)):
            yield DEDENT(token.lineno)


# The top-level filter adds an ENDMARKER, if requested.
# Python's grammar uses it.
def filter(lexer, add_endmarker=True):
    token = None
    tokens = iter(lexer.token, None)
    tokens = track_tokens_filter(lexer, tokens)
    for token in indentation_filter(tokens):
        yield token

    if add_endmarker:
        lineno = 1
        if token is not None:
            lineno = token.lineno
        yield _new_token("ENDMARKER", lineno)

# Combine Ply and my filters into a new lexer


class IndentLexer(object):

    def __init__(self, debug=0, optimize=0, lextab='lextab', reflags=0):
        self.lexer = lex.lex(debug=debug, optimize=optimize,
                             lextab=lextab, reflags=reflags)
        self.token_stream = None

    def input(self, s, add_endmarker=True):
        self.lexer.paren_count = 0
        self.lexer.input(s)
        self.token_stream = filter(self.lexer, add_endmarker)

    def token(self):
        try:
            return self.token_stream.next()
        except StopIteration:
            return None

##########   Parser (tokens -> AST) ######

# also part of Ply
#import yacc

# I use the Python AST
from compiler import ast

# Helper function


def Assign(left, right):
    names = []
    if isinstance(left, ast.Name):
        # Single assignment on left
        return ast.Assign([ast.AssName(left.name, 'OP_ASSIGN')], right)
    elif isinstance(left, ast.Tuple):
        # List of things - make sure they are Name nodes
        names = []
        for child in left.getChildren():
            if not isinstance(child, ast.Name):
                raise SyntaxError("that assignment not supported")
            names.append(child.name)
        ass_list = [ast.AssName(name, 'OP_ASSIGN') for name in names]
        return ast.Assign([ast.AssTuple(ass_list)], right)
    else:
        raise SyntaxError("Can't do that yet")


# The grammar comments come from Python's Grammar/Grammar file

# NB: compound_stmt in single_input is followed by extra NEWLINE!
# file_input: (NEWLINE | stmt)* ENDMARKER
def p_file_input_end(p):
    """file_input_end : file_input ENDMARKER"""
    p[0] = ast.Stmt(p[1])


def p_file_input(p):
    """file_input : file_input NEWLINE
                  | file_input stmt
                  | NEWLINE
                  | stmt"""
    if isinstance(p[len(p) - 1], basestring):
        if len(p) == 3:
            p[0] = p[1]
        else:
            p[0] = []  # p == 2 --> only a blank line
    else:
        if len(p) == 3:
            p[0] = p[1] + p[2]
        else:
            p[0] = p[1]


# funcdef: [decorators] 'def' NAME parameters ':' suite
# ignoring decorators
def p_funcdef(p):
    "funcdef : DEF NAME parameters COLON suite"
    p[0] = ast.Function(None, p[2], tuple(p[3]), (), 0, None, p[5])

# parameters: '(' [varargslist] ')'


def p_parameters(p):
    """parameters : LPAR RPAR
                  | LPAR varargslist RPAR"""
    if len(p) == 3:
        p[0] = []
    else:
        p[0] = p[2]


# varargslist: (fpdef ['=' test] ',')* ('*' NAME [',' '**' NAME] | '**' NAME) |
# highly simplified
def p_varargslist(p):
    """varargslist : varargslist COMMA NAME
                   | NAME"""
    if len(p) == 4:
        p[0] = p[1] + p[3]
    else:
        p[0] = [p[1]]

# stmt: simple_stmt | compound_stmt


def p_stmt_simple(p):
    """stmt : simple_stmt"""
    # simple_stmt is a list
    p[0] = p[1]


def p_stmt_compound(p):
    """stmt : compound_stmt"""
    p[0] = [p[1]]

# simple_stmt: small_stmt (';' small_stmt)* [';'] NEWLINE


def p_simple_stmt(p):
    """simple_stmt : small_stmts NEWLINE
                   | small_stmts SEMICOLON NEWLINE"""
    p[0] = p[1]


def p_small_stmts(p):
    """small_stmts : small_stmts SEMICOLON small_stmt
                   | small_stmt"""
    if len(p) == 4:
        p[0] = p[1] + [p[3]]
    else:
        p[0] = [p[1]]

# small_stmt: expr_stmt | print_stmt  | del_stmt | pass_stmt | flow_stmt |
#    import_stmt | global_stmt | exec_stmt | assert_stmt


def p_small_stmt(p):
    """small_stmt : flow_stmt
                  | expr_stmt"""
    p[0] = p[1]

# expr_stmt: testlist (augassign (yield_expr|testlist) |
#                      ('=' (yield_expr|testlist))*)
# augassign: ('+=' | '-=' | '*=' | '/=' | '%=' | '&=' | '|=' | '^=' |
#             '<<=' | '>>=' | '**=' | '//=')


def p_expr_stmt(p):
    """expr_stmt : testlist ASSIGN testlist
                 | testlist """
    if len(p) == 2:
        # a list of expressions
        p[0] = ast.Discard(p[1])
    else:
        p[0] = Assign(p[1], p[3])


def p_flow_stmt(p):
    "flow_stmt : return_stmt"
    p[0] = p[1]

# return_stmt: 'return' [testlist]


def p_return_stmt(p):
    "return_stmt : RETURN testlist"
    p[0] = ast.Return(p[2])


def p_compound_stmt(p):
    """compound_stmt : if_stmt
                     | funcdef"""
    p[0] = p[1]


def p_if_stmt(p):
    'if_stmt : IF test COLON suite'
    p[0] = ast.If([(p[2], p[4])], None)


def p_suite(p):
    """suite : simple_stmt
             | NEWLINE INDENT stmts DEDENT"""
    if len(p) == 2:
        p[0] = ast.Stmt(p[1])
    else:
        p[0] = ast.Stmt(p[3])


def p_stmts(p):
    """stmts : stmts stmt
             | stmt"""
    if len(p) == 3:
        p[0] = p[1] + p[2]
    else:
        p[0] = p[1]

# No using Python's approach because Ply supports precedence

# comparison: expr (comp_op expr)*
# arith_expr: term (('+'|'-') term)*
# term: factor (('*'|'/'|'%'|'//') factor)*
# factor: ('+'|'-'|'~') factor | power
# comp_op: '<'|'>'|'=='|'>='|'<='|'<>'|'!='|'in'|'not' 'in'|'is'|'is' 'not'


def make_lt_compare((left, right)):
    return ast.Compare(left, [('<', right), ])


def make_gt_compare((left, right)):
    return ast.Compare(left, [('>', right), ])


def make_eq_compare((left, right)):
    return ast.Compare(left, [('==', right), ])


binary_ops = {
    "+": ast.Add,
    "-": ast.Sub,
    "*": ast.Mul,
    "/": ast.Div,
    "<": make_lt_compare,
    ">": make_gt_compare,
    "==": make_eq_compare,
}
unary_ops = {
    "+": ast.UnaryAdd,
    "-": ast.UnarySub,
}
precedence = (
    ("left", "EQ", "GT", "LT"),
    ("left", "PLUS", "MINUS"),
    ("left", "MULT", "DIV"),
)


def p_comparison(p):
    """comparison : comparison PLUS comparison
                  | comparison MINUS comparison
                  | comparison MULT comparison
                  | comparison DIV comparison
                  | comparison LT comparison
                  | comparison EQ comparison
                  | comparison GT comparison
                  | PLUS comparison
                  | MINUS comparison
                  | power"""
    if len(p) == 4:
        p[0] = binary_ops[p[2]]((p[1], p[3]))
    elif len(p) == 3:
        p[0] = unary_ops[p[1]](p[2])
    else:
        p[0] = p[1]

# power: atom trailer* ['**' factor]
# trailers enables function calls.  I only allow one level of calls
# so this is 'trailer'


def p_power(p):
    """power : atom
             | atom trailer"""
    if len(p) == 2:
        p[0] = p[1]
    else:
        if p[2][0] == "CALL":
            p[0] = ast.CallFunc(p[1], p[2][1], None, None)
        else:
            raise AssertionError("not implemented")


def p_atom_name(p):
    """atom : NAME"""
    p[0] = ast.Name(p[1])


def p_atom_number(p):
    """atom : NUMBER
            | STRING"""
    p[0] = ast.Const(p[1])


def p_atom_tuple(p):
    """atom : LPAR testlist RPAR"""
    p[0] = p[2]

# trailer: '(' [arglist] ')' | '[' subscriptlist ']' | '.' NAME


def p_trailer(p):
    "trailer : LPAR arglist RPAR"
    p[0] = ("CALL", p[2])

# testlist: test (',' test)* [',']
# Contains shift/reduce error


def p_testlist(p):
    """testlist : testlist_multi COMMA
                | testlist_multi """
    if len(p) == 2:
        p[0] = p[1]
    else:
        # May need to promote singleton to tuple
        if isinstance(p[1], list):
            p[0] = p[1]
        else:
            p[0] = [p[1]]
    # Convert into a tuple?
    if isinstance(p[0], list):
        p[0] = ast.Tuple(p[0])


def p_testlist_multi(p):
    """testlist_multi : testlist_multi COMMA test
                      | test"""
    if len(p) == 2:
        # singleton
        p[0] = p[1]
    else:
        if isinstance(p[1], list):
            p[0] = p[1] + [p[3]]
        else:
            # singleton -> tuple
            p[0] = [p[1], p[3]]


# test: or_test ['if' or_test 'else' test] | lambdef
#  as I don't support 'and', 'or', and 'not' this works down to 'comparison'
def p_test(p):
    "test : comparison"
    p[0] = p[1]


# arglist: (argument ',')* (argument [',']| '*' test [',' '**' test] | '**' test)
# XXX INCOMPLETE: this doesn't allow the trailing comma
def p_arglist(p):
    """arglist : arglist COMMA argument
               | argument"""
    if len(p) == 4:
        p[0] = p[1] + [p[3]]
    else:
        p[0] = [p[1]]

# argument: test [gen_for] | test '=' test  # Really [keyword '='] test


def p_argument(p):
    "argument : test"
    p[0] = p[1]


def p_error(p):
    # print "Error!", repr(p)
    raise SyntaxError(p)


class GardenSnakeParser(object):

    def __init__(self, lexer=None):
        if lexer is None:
            lexer = IndentLexer()
        self.lexer = lexer
        self.parser = yacc.yacc(start="file_input_end")

    def parse(self, code):
        self.lexer.input(code)
        result = self.parser.parse(lexer=self.lexer)
        return ast.Module(None, result)


###### Code generation ######

from compiler import misc, syntax, pycodegen


class GardenSnakeCompiler(object):

    def __init__(self):
        self.parser = GardenSnakeParser()

    def compile(self, code, filename="<string>"):
        tree = self.parser.parse(code)
        # print  tree
        misc.set_filename(filename, tree)
        syntax.check(tree)
        gen = pycodegen.ModuleCodeGenerator(tree)
        code = gen.getCode()
        return code

####### Test code #######

compile = GardenSnakeCompiler().compile

code = r"""

print('LET\'S TRY THIS \\OUT')
  
#Comment here
def x(a):
    print('called with',a)
    if a == 1:
        return 2
    if a*2 > 10: return 999 / 4
        # Another comment here

    return a+2*3

ints = (1, 2,
   3, 4,
5)
print('mutiline-expression', ints)

t = 4+1/3*2+6*(9-5+1)
print('predence test; should be 34+2/3:', t, t==(34+2/3))

print('numbers', 1,2,3,4,5)
if 1:
 8
 a=9
 print(x(a))

print(x(1))
print(x(2))
print(x(8),'3')
print('this is decimal', 1/5)
print('BIG DECIMAL', 1.234567891234567e12345)

"""

# Set up the GardenSnake run-time environment


def print_(*args):
    print "-->", " ".join(map(str, args))

globals()["print"] = print_

compiled_code = compile(code)

exec compiled_code in globals()
print "Done"
