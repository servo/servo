# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import fileinput
import re
import random


def is_comment(line):
    return re.search(r"\/\/.*", line)


def init_variables(if_blocks):
    random_index = random.randint(0, len(if_blocks) - 1)
    start_counter = 0
    end_counter = 0
    lines_to_delete = []
    line_to_mutate = if_blocks[random_index]
    return random_index, start_counter, end_counter, lines_to_delete, line_to_mutate


def deleteStatements(file_name, line_numbers):
    for line in fileinput.input(file_name, inplace=True):
        if fileinput.lineno() not in line_numbers:
            print line.rstrip()


class Strategy:
    def __init__(self):
        self._strategy_name = ""
        self._replace_strategy = {}

    def mutate(self, file_name):
        line_numbers = []
        for line in fileinput.input(file_name):
            if not is_comment(line) and re.search(self._replace_strategy['regex'], line):
                line_numbers.append(fileinput.lineno())
        if len(line_numbers) == 0:
            return -1
        else:
            mutation_line_number = line_numbers[random.randint(0, len(line_numbers) - 1)]
            for line in fileinput.input(file_name, inplace=True):
                if fileinput.lineno() == mutation_line_number:
                    line = re.sub(self._replace_strategy['regex'], self._replace_strategy['replaceString'], line)
                print line.rstrip()
            return mutation_line_number


class AndOr(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        logical_and = r"(?<=\s)&&(?=\s)"
        self._replace_strategy = {
            'regex': logical_and,
            'replaceString': '||'
        }


class IfTrue(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        if_condition = r"(?<=if\s)(.*)(?=\s\{)"
        self._replace_strategy = {
            'regex': if_condition,
            'replaceString': 'true'
        }


class IfFalse(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        if_condition = r"(?<=if\s)(.*)(?=\s\{)"
        self._replace_strategy = {
            'regex': if_condition,
            'replaceString': 'false'
        }


class ModifyComparision(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        less_than_equals = r"(?<=\s)(\<)\=(?=\s)"
        greater_than_equals = r"(?<=\s)(\<)\=(?=\s)"
        self._replace_strategy = {
            'regex': (less_than_equals + '|' + greater_than_equals),
            'replaceString': r"\1"
        }


class MinusToPlus(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        arithmetic_minus = r"(?<=\s)\-(?=\s.+)"
        minus_in_shorthand = r"(?<=\s)\-(?=\=)"
        self._replace_strategy = {
            'regex': (arithmetic_minus + '|' + minus_in_shorthand),
            'replaceString': '+'
        }


class PlusToMinus(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        arithmetic_plus = r"(?<=[^\"]\s)\+(?=\s[^A-Z\'?\":\{]+)"
        plus_in_shorthand = r"(?<=\s)\+(?=\=)"
        self._replace_strategy = {
            'regex': (arithmetic_plus + '|' + plus_in_shorthand),
            'replaceString': '-'
        }


class AtomicString(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        string_literal = r"(?<=\").+(?=\")"
        self._replace_strategy = {
            'regex': string_literal,
            'replaceString': ' '
        }


class DuplicateLine(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        self._strategy_name = "duplicate"
        append_statement = r".+?append\(.+?\).*?;"
        remove_statement = r".+?remove\(.*?\).*?;"
        push_statement = r".+?push\(.+?\).*?;"
        pop_statement = r".+?pop\(.+?\).*?;"
        plus_equals_statement = r".+?\s\+\=\s.*"
        minus_equals_statement = r".+?\s\-\=\s.*"
        self._replace_strategy = {
            'regex': (append_statement + '|' + remove_statement + '|' + push_statement +
                      '|' + pop_statement + '|' + plus_equals_statement + '|' + minus_equals_statement),
            'replaceString': r"\g<0>\n\g<0>",
        }


class DeleteIfBlock(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        self.if_block = r"^\s+if\s(.+)\s\{"
        self.else_block = r"\selse(.+)\{"

    def mutate(self, file_name):
        code_lines = []
        if_blocks = []
        for line in fileinput.input(file_name):
            code_lines.append(line)
            if re.search(self.if_block, line):
                if_blocks.append(fileinput.lineno())
        if len(if_blocks) == 0:
            return -1
        random_index, start_counter, end_counter, lines_to_delete, line_to_mutate = init_variables(if_blocks)
        while line_to_mutate <= len(code_lines):
            current_line = code_lines[line_to_mutate - 1]
            next_line = code_lines[line_to_mutate]
            if re.search(self.else_block, current_line) is not None \
                    or re.search(self.else_block, next_line) is not None:
                if_blocks.pop(random_index)
                if len(if_blocks) == 0:
                    return -1
                else:
                    random_index, start_counter, end_counter, lines_to_delete, line_to_mutate = \
                        init_variables(if_blocks)
                    continue
            lines_to_delete.append(line_to_mutate)
            for ch in current_line:
                if ch == "{":
                    start_counter += 1
                elif ch == "}":
                    end_counter += 1
                if start_counter and start_counter == end_counter:
                    deleteStatements(file_name, lines_to_delete)
                    return lines_to_delete[0]
            line_to_mutate += 1


def get_strategies():
    return AndOr, IfTrue, IfFalse, ModifyComparision, PlusToMinus, MinusToPlus, \
        AtomicString, DuplicateLine, DeleteIfBlock


class Mutator:
    def __init__(self, strategy):
        self._strategy = strategy

    def mutate(self, file_name):
        return self._strategy.mutate(file_name)
