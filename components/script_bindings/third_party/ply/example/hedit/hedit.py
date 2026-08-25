# -----------------------------------------------------------------------------
# hedit.py
#
# Paring of Fortran H Edit descriptions (Contributed by Pearu Peterson)
#
# These tokens can't be easily tokenized because they are of the following
# form:
#
#   nHc1...cn
#
# where n is a positive integer and c1 ... cn are characters.
#
# This example shows how to modify the state of the lexer to parse
# such tokens
# -----------------------------------------------------------------------------

import sys
sys.path.insert(0, "../..")


tokens = (
    'H_EDIT_DESCRIPTOR',
)

# Tokens
t_ignore = " \t\n"


def t_H_EDIT_DESCRIPTOR(t):
    r"\d+H.*"                     # This grabs all of the remaining text
    i = t.value.index('H')
    n = eval(t.value[:i])

    # Adjust the tokenizing position
    t.lexer.lexpos -= len(t.value) - (i + 1 + n)

    t.value = t.value[i + 1:i + 1 + n]
    return t


def t_error(t):
    print("Illegal character '%s'" % t.value[0])
    t.lexer.skip(1)

# Build the lexer
import ply.lex as lex
lex.lex()
lex.runmain()
