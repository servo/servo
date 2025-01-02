# lexer for yacc-grammars
#
# Author: David Beazley (dave@dabeaz.com)
# Date  : October 2, 2006

import sys
sys.path.append("../..")

from ply import *

tokens = (
    'LITERAL', 'SECTION', 'TOKEN', 'LEFT', 'RIGHT', 'PREC', 'START', 'TYPE', 'NONASSOC', 'UNION', 'CODE',
    'ID', 'QLITERAL', 'NUMBER',
)

states = (('code', 'exclusive'),)

literals = [';', ',', '<', '>', '|', ':']
t_ignore = ' \t'

t_TOKEN = r'%token'
t_LEFT = r'%left'
t_RIGHT = r'%right'
t_NONASSOC = r'%nonassoc'
t_PREC = r'%prec'
t_START = r'%start'
t_TYPE = r'%type'
t_UNION = r'%union'
t_ID = r'[a-zA-Z_][a-zA-Z_0-9]*'
t_QLITERAL  = r'''(?P<quote>['"]).*?(?P=quote)'''
t_NUMBER = r'\d+'


def t_SECTION(t):
    r'%%'
    if getattr(t.lexer, "lastsection", 0):
        t.value = t.lexer.lexdata[t.lexpos + 2:]
        t.lexer.lexpos = len(t.lexer.lexdata)
    else:
        t.lexer.lastsection = 0
    return t

# Comments


def t_ccomment(t):
    r'/\*(.|\n)*?\*/'
    t.lexer.lineno += t.value.count('\n')

t_ignore_cppcomment = r'//.*'


def t_LITERAL(t):
    r'%\{(.|\n)*?%\}'
    t.lexer.lineno += t.value.count("\n")
    return t


def t_NEWLINE(t):
    r'\n'
    t.lexer.lineno += 1


def t_code(t):
    r'\{'
    t.lexer.codestart = t.lexpos
    t.lexer.level = 1
    t.lexer.begin('code')


def t_code_ignore_string(t):
    r'\"([^\\\n]|(\\.))*?\"'


def t_code_ignore_char(t):
    r'\'([^\\\n]|(\\.))*?\''


def t_code_ignore_comment(t):
    r'/\*(.|\n)*?\*/'


def t_code_ignore_cppcom(t):
    r'//.*'


def t_code_lbrace(t):
    r'\{'
    t.lexer.level += 1


def t_code_rbrace(t):
    r'\}'
    t.lexer.level -= 1
    if t.lexer.level == 0:
        t.type = 'CODE'
        t.value = t.lexer.lexdata[t.lexer.codestart:t.lexpos + 1]
        t.lexer.begin('INITIAL')
        t.lexer.lineno += t.value.count('\n')
        return t

t_code_ignore_nonspace = r'[^\s\}\'\"\{]+'
t_code_ignore_whitespace = r'\s+'
t_code_ignore = ""


def t_code_error(t):
    raise RuntimeError


def t_error(t):
    print("%d: Illegal character '%s'" % (t.lexer.lineno, t.value[0]))
    print(t.value)
    t.lexer.skip(1)

lex.lex()

if __name__ == '__main__':
    lex.runmain()
