# -----------------------------------------------------------------------------
# calc.py
#
# A simple calculator with variables.   This is from O'Reilly's
# "Lex and Yacc", p. 63.
# -----------------------------------------------------------------------------

import sys
sys.path.insert(0, "../..")

if sys.version_info[0] >= 3:
    raw_input = input

tokens = (
    'NAME', 'NUMBER',
    'PLUS', 'MINUS', 'TIMES', 'DIVIDE', 'EQUALS',
    'LPAREN', 'RPAREN',
)

# Tokens

t_PLUS = r'\+'
t_MINUS = r'-'
t_TIMES = r'\*'
t_DIVIDE = r'/'
t_EQUALS = r'='
t_LPAREN = r'\('
t_RPAREN = r'\)'
t_NAME = r'[a-zA-Z_][a-zA-Z0-9_]*'


def t_NUMBER(t):
    r'\d+'
    try:
        t.value = int(t.value)
    except ValueError:
        print("Integer value too large %s" % t.value)
        t.value = 0
    return t

t_ignore = " \t"


def t_newline(t):
    r'\n+'
    t.lexer.lineno += t.value.count("\n")


def t_error(t):
    print("Illegal character '%s'" % t.value[0])
    t.lexer.skip(1)

# Build the lexer
import ply.lex as lex
lex.lex(optimize=1)

# Parsing rules

precedence = (
    ('left', 'PLUS', 'MINUS'),
    ('left', 'TIMES', 'DIVIDE'),
    ('right', 'UMINUS'),
)

# dictionary of names
names = {}


def p_statement_assign(t):
    'statement : NAME EQUALS expression'
    names[t[1]] = t[3]


def p_statement_expr(t):
    'statement : expression'
    print(t[1])


def p_expression_binop(t):
    '''expression : expression PLUS expression
                  | expression MINUS expression
                  | expression TIMES expression
                  | expression DIVIDE expression'''
    if t[2] == '+':
        t[0] = t[1] + t[3]
    elif t[2] == '-':
        t[0] = t[1] - t[3]
    elif t[2] == '*':
        t[0] = t[1] * t[3]
    elif t[2] == '/':
        t[0] = t[1] / t[3]
    elif t[2] == '<':
        t[0] = t[1] < t[3]


def p_expression_uminus(t):
    'expression : MINUS expression %prec UMINUS'
    t[0] = -t[2]


def p_expression_group(t):
    'expression : LPAREN expression RPAREN'
    t[0] = t[2]


def p_expression_number(t):
    'expression : NUMBER'
    t[0] = t[1]


def p_expression_name(t):
    'expression : NAME'
    try:
        t[0] = names[t[1]]
    except LookupError:
        print("Undefined name '%s'" % t[1])
        t[0] = 0


def p_error(t):
    if t:
        print("Syntax error at '%s'" % t.value)
    else:
        print("Syntax error at EOF")

import ply.yacc as yacc
yacc.yacc(optimize=1)

while 1:
    try:
        s = raw_input('calc > ')
    except EOFError:
        break
    yacc.parse(s)
