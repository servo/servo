# An implementation of Dartmouth BASIC (1964)
#

from ply import *
import basiclex

tokens = basiclex.tokens

precedence = (
    ('left', 'PLUS', 'MINUS'),
    ('left', 'TIMES', 'DIVIDE'),
    ('left', 'POWER'),
    ('right', 'UMINUS')
)

# A BASIC program is a series of statements.  We represent the program as a
# dictionary of tuples indexed by line number.


def p_program(p):
    '''program : program statement
               | statement'''

    if len(p) == 2 and p[1]:
        p[0] = {}
        line, stat = p[1]
        p[0][line] = stat
    elif len(p) == 3:
        p[0] = p[1]
        if not p[0]:
            p[0] = {}
        if p[2]:
            line, stat = p[2]
            p[0][line] = stat

# This catch-all rule is used for any catastrophic errors.  In this case,
# we simply return nothing


def p_program_error(p):
    '''program : error'''
    p[0] = None
    p.parser.error = 1

# Format of all BASIC statements.


def p_statement(p):
    '''statement : INTEGER command NEWLINE'''
    if isinstance(p[2], str):
        print("%s %s %s" % (p[2], "AT LINE", p[1]))
        p[0] = None
        p.parser.error = 1
    else:
        lineno = int(p[1])
        p[0] = (lineno, p[2])

# Interactive statements.


def p_statement_interactive(p):
    '''statement : RUN NEWLINE
                 | LIST NEWLINE
                 | NEW NEWLINE'''
    p[0] = (0, (p[1], 0))

# Blank line number


def p_statement_blank(p):
    '''statement : INTEGER NEWLINE'''
    p[0] = (0, ('BLANK', int(p[1])))

# Error handling for malformed statements


def p_statement_bad(p):
    '''statement : INTEGER error NEWLINE'''
    print("MALFORMED STATEMENT AT LINE %s" % p[1])
    p[0] = None
    p.parser.error = 1

# Blank line


def p_statement_newline(p):
    '''statement : NEWLINE'''
    p[0] = None

# LET statement


def p_command_let(p):
    '''command : LET variable EQUALS expr'''
    p[0] = ('LET', p[2], p[4])


def p_command_let_bad(p):
    '''command : LET variable EQUALS error'''
    p[0] = "BAD EXPRESSION IN LET"

# READ statement


def p_command_read(p):
    '''command : READ varlist'''
    p[0] = ('READ', p[2])


def p_command_read_bad(p):
    '''command : READ error'''
    p[0] = "MALFORMED VARIABLE LIST IN READ"

# DATA statement


def p_command_data(p):
    '''command : DATA numlist'''
    p[0] = ('DATA', p[2])


def p_command_data_bad(p):
    '''command : DATA error'''
    p[0] = "MALFORMED NUMBER LIST IN DATA"

# PRINT statement


def p_command_print(p):
    '''command : PRINT plist optend'''
    p[0] = ('PRINT', p[2], p[3])


def p_command_print_bad(p):
    '''command : PRINT error'''
    p[0] = "MALFORMED PRINT STATEMENT"

# Optional ending on PRINT. Either a comma (,) or semicolon (;)


def p_optend(p):
    '''optend : COMMA 
              | SEMI
              |'''
    if len(p) == 2:
        p[0] = p[1]
    else:
        p[0] = None

# PRINT statement with no arguments


def p_command_print_empty(p):
    '''command : PRINT'''
    p[0] = ('PRINT', [], None)

# GOTO statement


def p_command_goto(p):
    '''command : GOTO INTEGER'''
    p[0] = ('GOTO', int(p[2]))


def p_command_goto_bad(p):
    '''command : GOTO error'''
    p[0] = "INVALID LINE NUMBER IN GOTO"

# IF-THEN statement


def p_command_if(p):
    '''command : IF relexpr THEN INTEGER'''
    p[0] = ('IF', p[2], int(p[4]))


def p_command_if_bad(p):
    '''command : IF error THEN INTEGER'''
    p[0] = "BAD RELATIONAL EXPRESSION"


def p_command_if_bad2(p):
    '''command : IF relexpr THEN error'''
    p[0] = "INVALID LINE NUMBER IN THEN"

# FOR statement


def p_command_for(p):
    '''command : FOR ID EQUALS expr TO expr optstep'''
    p[0] = ('FOR', p[2], p[4], p[6], p[7])


def p_command_for_bad_initial(p):
    '''command : FOR ID EQUALS error TO expr optstep'''
    p[0] = "BAD INITIAL VALUE IN FOR STATEMENT"


def p_command_for_bad_final(p):
    '''command : FOR ID EQUALS expr TO error optstep'''
    p[0] = "BAD FINAL VALUE IN FOR STATEMENT"


def p_command_for_bad_step(p):
    '''command : FOR ID EQUALS expr TO expr STEP error'''
    p[0] = "MALFORMED STEP IN FOR STATEMENT"

# Optional STEP qualifier on FOR statement


def p_optstep(p):
    '''optstep : STEP expr
               | empty'''
    if len(p) == 3:
        p[0] = p[2]
    else:
        p[0] = None

# NEXT statement


def p_command_next(p):
    '''command : NEXT ID'''

    p[0] = ('NEXT', p[2])


def p_command_next_bad(p):
    '''command : NEXT error'''
    p[0] = "MALFORMED NEXT"

# END statement


def p_command_end(p):
    '''command : END'''
    p[0] = ('END',)

# REM statement


def p_command_rem(p):
    '''command : REM'''
    p[0] = ('REM', p[1])

# STOP statement


def p_command_stop(p):
    '''command : STOP'''
    p[0] = ('STOP',)

# DEF statement


def p_command_def(p):
    '''command : DEF ID LPAREN ID RPAREN EQUALS expr'''
    p[0] = ('FUNC', p[2], p[4], p[7])


def p_command_def_bad_rhs(p):
    '''command : DEF ID LPAREN ID RPAREN EQUALS error'''
    p[0] = "BAD EXPRESSION IN DEF STATEMENT"


def p_command_def_bad_arg(p):
    '''command : DEF ID LPAREN error RPAREN EQUALS expr'''
    p[0] = "BAD ARGUMENT IN DEF STATEMENT"

# GOSUB statement


def p_command_gosub(p):
    '''command : GOSUB INTEGER'''
    p[0] = ('GOSUB', int(p[2]))


def p_command_gosub_bad(p):
    '''command : GOSUB error'''
    p[0] = "INVALID LINE NUMBER IN GOSUB"

# RETURN statement


def p_command_return(p):
    '''command : RETURN'''
    p[0] = ('RETURN',)

# DIM statement


def p_command_dim(p):
    '''command : DIM dimlist'''
    p[0] = ('DIM', p[2])


def p_command_dim_bad(p):
    '''command : DIM error'''
    p[0] = "MALFORMED VARIABLE LIST IN DIM"

# List of variables supplied to DIM statement


def p_dimlist(p):
    '''dimlist : dimlist COMMA dimitem
               | dimitem'''
    if len(p) == 4:
        p[0] = p[1]
        p[0].append(p[3])
    else:
        p[0] = [p[1]]

# DIM items


def p_dimitem_single(p):
    '''dimitem : ID LPAREN INTEGER RPAREN'''
    p[0] = (p[1], eval(p[3]), 0)


def p_dimitem_double(p):
    '''dimitem : ID LPAREN INTEGER COMMA INTEGER RPAREN'''
    p[0] = (p[1], eval(p[3]), eval(p[5]))

# Arithmetic expressions


def p_expr_binary(p):
    '''expr : expr PLUS expr
            | expr MINUS expr
            | expr TIMES expr
            | expr DIVIDE expr
            | expr POWER expr'''

    p[0] = ('BINOP', p[2], p[1], p[3])


def p_expr_number(p):
    '''expr : INTEGER
            | FLOAT'''
    p[0] = ('NUM', eval(p[1]))


def p_expr_variable(p):
    '''expr : variable'''
    p[0] = ('VAR', p[1])


def p_expr_group(p):
    '''expr : LPAREN expr RPAREN'''
    p[0] = ('GROUP', p[2])


def p_expr_unary(p):
    '''expr : MINUS expr %prec UMINUS'''
    p[0] = ('UNARY', '-', p[2])

# Relational expressions


def p_relexpr(p):
    '''relexpr : expr LT expr
               | expr LE expr
               | expr GT expr
               | expr GE expr
               | expr EQUALS expr
               | expr NE expr'''
    p[0] = ('RELOP', p[2], p[1], p[3])

# Variables


def p_variable(p):
    '''variable : ID
              | ID LPAREN expr RPAREN
              | ID LPAREN expr COMMA expr RPAREN'''
    if len(p) == 2:
        p[0] = (p[1], None, None)
    elif len(p) == 5:
        p[0] = (p[1], p[3], None)
    else:
        p[0] = (p[1], p[3], p[5])

# Builds a list of variable targets as a Python list


def p_varlist(p):
    '''varlist : varlist COMMA variable
               | variable'''
    if len(p) > 2:
        p[0] = p[1]
        p[0].append(p[3])
    else:
        p[0] = [p[1]]


# Builds a list of numbers as a Python list

def p_numlist(p):
    '''numlist : numlist COMMA number
               | number'''

    if len(p) > 2:
        p[0] = p[1]
        p[0].append(p[3])
    else:
        p[0] = [p[1]]

# A number. May be an integer or a float


def p_number(p):
    '''number  : INTEGER
               | FLOAT'''
    p[0] = eval(p[1])

# A signed number.


def p_number_signed(p):
    '''number  : MINUS INTEGER
               | MINUS FLOAT'''
    p[0] = eval("-" + p[2])

# List of targets for a print statement
# Returns a list of tuples (label,expr)


def p_plist(p):
    '''plist   : plist COMMA pitem
               | pitem'''
    if len(p) > 3:
        p[0] = p[1]
        p[0].append(p[3])
    else:
        p[0] = [p[1]]


def p_item_string(p):
    '''pitem : STRING'''
    p[0] = (p[1][1:-1], None)


def p_item_string_expr(p):
    '''pitem : STRING expr'''
    p[0] = (p[1][1:-1], p[2])


def p_item_expr(p):
    '''pitem : expr'''
    p[0] = ("", p[1])

# Empty


def p_empty(p):
    '''empty : '''

# Catastrophic error handler


def p_error(p):
    if not p:
        print("SYNTAX ERROR AT EOF")

bparser = yacc.yacc()


def parse(data, debug=0):
    bparser.error = 0
    p = bparser.parse(data, debug=debug)
    if bparser.error:
        return None
    return p
