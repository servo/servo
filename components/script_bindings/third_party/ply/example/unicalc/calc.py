# -----------------------------------------------------------------------------
# calc.py
#
# A simple calculator with variables.   This is from O'Reilly's
# "Lex and Yacc", p. 63.
#
# This example uses unicode strings for tokens, docstrings, and input.
# -----------------------------------------------------------------------------

import sys
sys.path.insert(0, "../..")

tokens = (
    'NAME', 'NUMBER',
    'PLUS', 'MINUS', 'TIMES', 'DIVIDE', 'EQUALS',
    'LPAREN', 'RPAREN',
)

# Tokens

t_PLUS = ur'\+'
t_MINUS = ur'-'
t_TIMES = ur'\*'
t_DIVIDE = ur'/'
t_EQUALS = ur'='
t_LPAREN = ur'\('
t_RPAREN = ur'\)'
t_NAME = ur'[a-zA-Z_][a-zA-Z0-9_]*'


def t_NUMBER(t):
    ur'\d+'
    try:
        t.value = int(t.value)
    except ValueError:
        print "Integer value too large", t.value
        t.value = 0
    return t

t_ignore = u" \t"


def t_newline(t):
    ur'\n+'
    t.lexer.lineno += t.value.count("\n")


def t_error(t):
    print "Illegal character '%s'" % t.value[0]
    t.lexer.skip(1)

# Build the lexer
import ply.lex as lex
lex.lex()

# Parsing rules

precedence = (
    ('left', 'PLUS', 'MINUS'),
    ('left', 'TIMES', 'DIVIDE'),
    ('right', 'UMINUS'),
)

# dictionary of names
names = {}


def p_statement_assign(p):
    'statement : NAME EQUALS expression'
    names[p[1]] = p[3]


def p_statement_expr(p):
    'statement : expression'
    print p[1]


def p_expression_binop(p):
    '''expression : expression PLUS expression
                  | expression MINUS expression
                  | expression TIMES expression
                  | expression DIVIDE expression'''
    if p[2] == u'+':
        p[0] = p[1] + p[3]
    elif p[2] == u'-':
        p[0] = p[1] - p[3]
    elif p[2] == u'*':
        p[0] = p[1] * p[3]
    elif p[2] == u'/':
        p[0] = p[1] / p[3]


def p_expression_uminus(p):
    'expression : MINUS expression %prec UMINUS'
    p[0] = -p[2]


def p_expression_group(p):
    'expression : LPAREN expression RPAREN'
    p[0] = p[2]


def p_expression_number(p):
    'expression : NUMBER'
    p[0] = p[1]


def p_expression_name(p):
    'expression : NAME'
    try:
        p[0] = names[p[1]]
    except LookupError:
        print "Undefined name '%s'" % p[1]
        p[0] = 0


def p_error(p):
    if p:
        print "Syntax error at '%s'" % p.value
    else:
        print "Syntax error at EOF"

import ply.yacc as yacc
yacc.yacc()

while 1:
    try:
        s = raw_input('calc > ')
    except EOFError:
        break
    if not s:
        continue
    yacc.parse(unicode(s))
