# parser for Unix yacc-based grammars
#
# Author: David Beazley (dave@dabeaz.com)
# Date  : October 2, 2006

import ylex
tokens = ylex.tokens

from ply import *

tokenlist = []
preclist = []

emit_code = 1


def p_yacc(p):
    '''yacc : defsection rulesection'''


def p_defsection(p):
    '''defsection : definitions SECTION
                  | SECTION'''
    p.lexer.lastsection = 1
    print("tokens = ", repr(tokenlist))
    print()
    print("precedence = ", repr(preclist))
    print()
    print("# -------------- RULES ----------------")
    print()


def p_rulesection(p):
    '''rulesection : rules SECTION'''

    print("# -------------- RULES END ----------------")
    print_code(p[2], 0)


def p_definitions(p):
    '''definitions : definitions definition
                   | definition'''


def p_definition_literal(p):
    '''definition : LITERAL'''
    print_code(p[1], 0)


def p_definition_start(p):
    '''definition : START ID'''
    print("start = '%s'" % p[2])


def p_definition_token(p):
    '''definition : toktype opttype idlist optsemi '''
    for i in p[3]:
        if i[0] not in "'\"":
            tokenlist.append(i)
    if p[1] == '%left':
        preclist.append(('left',) + tuple(p[3]))
    elif p[1] == '%right':
        preclist.append(('right',) + tuple(p[3]))
    elif p[1] == '%nonassoc':
        preclist.append(('nonassoc',) + tuple(p[3]))


def p_toktype(p):
    '''toktype : TOKEN
               | LEFT
               | RIGHT
               | NONASSOC'''
    p[0] = p[1]


def p_opttype(p):
    '''opttype : '<' ID '>'
               | empty'''


def p_idlist(p):
    '''idlist  : idlist optcomma tokenid
               | tokenid'''
    if len(p) == 2:
        p[0] = [p[1]]
    else:
        p[0] = p[1]
        p[1].append(p[3])


def p_tokenid(p):
    '''tokenid : ID 
               | ID NUMBER
               | QLITERAL
               | QLITERAL NUMBER'''
    p[0] = p[1]


def p_optsemi(p):
    '''optsemi : ';'
               | empty'''


def p_optcomma(p):
    '''optcomma : ','
                | empty'''


def p_definition_type(p):
    '''definition : TYPE '<' ID '>' namelist optsemi'''
    # type declarations are ignored


def p_namelist(p):
    '''namelist : namelist optcomma ID
                | ID'''


def p_definition_union(p):
    '''definition : UNION CODE optsemi'''
    # Union declarations are ignored


def p_rules(p):
    '''rules   : rules rule
               | rule'''
    if len(p) == 2:
        rule = p[1]
    else:
        rule = p[2]

    # Print out a Python equivalent of this rule

    embedded = []      # Embedded actions (a mess)
    embed_count = 0

    rulename = rule[0]
    rulecount = 1
    for r in rule[1]:
        # r contains one of the rule possibilities
        print("def p_%s_%d(p):" % (rulename, rulecount))
        prod = []
        prodcode = ""
        for i in range(len(r)):
            item = r[i]
            if item[0] == '{':    # A code block
                if i == len(r) - 1:
                    prodcode = item
                    break
                else:
                    # an embedded action
                    embed_name = "_embed%d_%s" % (embed_count, rulename)
                    prod.append(embed_name)
                    embedded.append((embed_name, item))
                    embed_count += 1
            else:
                prod.append(item)
        print("    '''%s : %s'''" % (rulename, " ".join(prod)))
        # Emit code
        print_code(prodcode, 4)
        print()
        rulecount += 1

    for e, code in embedded:
        print("def p_%s(p):" % e)
        print("    '''%s : '''" % e)
        print_code(code, 4)
        print()


def p_rule(p):
    '''rule : ID ':' rulelist ';' '''
    p[0] = (p[1], [p[3]])


def p_rule2(p):
    '''rule : ID ':' rulelist morerules ';' '''
    p[4].insert(0, p[3])
    p[0] = (p[1], p[4])


def p_rule_empty(p):
    '''rule : ID ':' ';' '''
    p[0] = (p[1], [[]])


def p_rule_empty2(p):
    '''rule : ID ':' morerules ';' '''

    p[3].insert(0, [])
    p[0] = (p[1], p[3])


def p_morerules(p):
    '''morerules : morerules '|' rulelist
                 | '|' rulelist
                 | '|'  '''

    if len(p) == 2:
        p[0] = [[]]
    elif len(p) == 3:
        p[0] = [p[2]]
    else:
        p[0] = p[1]
        p[0].append(p[3])

#   print("morerules", len(p), p[0])


def p_rulelist(p):
    '''rulelist : rulelist ruleitem
                | ruleitem'''

    if len(p) == 2:
        p[0] = [p[1]]
    else:
        p[0] = p[1]
        p[1].append(p[2])


def p_ruleitem(p):
    '''ruleitem : ID
                | QLITERAL
                | CODE
                | PREC'''
    p[0] = p[1]


def p_empty(p):
    '''empty : '''


def p_error(p):
    pass

yacc.yacc(debug=0)


def print_code(code, indent):
    if not emit_code:
        return
    codelines = code.splitlines()
    for c in codelines:
        print("%s# %s" % (" " * indent, c))
