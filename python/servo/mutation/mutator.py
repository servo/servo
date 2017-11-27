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
    return re.search(r'\/\/.*', line)


class Strategy:
    def __init__(self):
        self._replace_strategy = {}
        self._strategy_name = ""
        self._delete_strategy = {}

    def mutate(self, file_name):
        if self._strategy_name == "delete_if_statement":
            return self.mutate_random_block(file_name)
        else:
            return self.mutate_random_line(file_name)

    def deleteStatements(self, file_name, line_numbers):
        for line in fileinput.input(file_name, inplace=True):
            if fileinput.lineno() not in line_numbers:
                print line.rstrip()

    def init_variables(self, if_blocks, start_counter,end_counter,block_to_delete,idx_to_del):
        random_index = random.randint(0, len(if_blocks) - 1)
        if_blocks.pop(random_index)
        start_counter = 0
        end_counter = 0
        block_to_delete = ""
        idx_to_del=[]
        mutation_line_number = if_blocks[random_index] - 1
        return(random_index, if_blocks, start_counter, end_counter,block_to_delete,idx_to_del,mutation_line_number)

    def mutate_random_line(self, file_name):
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
                    if self._replace_strategy['replaceString']:
                        line = re.sub(self._replace_strategy['regex'], self._replace_strategy['replaceString'], line)
                    else:
                        if self._strategy_name == "duplicate":
                            replacement = line + line
                            line = re.sub(self._replace_strategy['regex'], replacement, line)
                print line.rstrip()
            return mutation_line_number

    def mutate_random_block(self, file_name):
        code_lines = []
        if_blocks = []
        random_index = 0
        start_counter = 0
        end_counter = 0
        block_to_delete = ""
        idx_to_del=[]
        mutation_line_number=0
        x=0  
        for line in fileinput.input(file_name):
                code_lines.append(line)
                if re.search(self._delete_strategy['ifBlock'], line):
                    if_blocks.append(fileinput.lineno())
        if len(if_blocks) == 0:
            return -1
        random_index, if_blocks, start_counter, end_counter,block_to_delete,idx_to_del, mutation_line_number = self.init_variables(if_blocks, start_counter,end_counter,block_to_delete,idx_to_del)
        while (mutation_line_number < len(code_lines)):
            current_line = code_lines[mutation_line_number]
            next_line = code_lines[mutation_line_number+1]
            if re.search(self._delete_strategy['elseBlock'], current_line) is not None or re.search(self._delete_strategy['elseBlock'], next_line) is not None:
                random_index, if_blocks, start_counter, end_counter,block_to_delete,idx_to_del,mutation_line_number = self.init_variables(if_blocks, start_counter,end_counter,block_to_delete,idx_to_del)
                if len(if_blocks) == 0:
                    return -1
                else:
                    continue
            idx_to_del.append(mutation_line_number+1)
            for ch in current_line:
                block_to_delete += ch
                if ch == "{":
                    start_counter += 1
                elif ch == "}":
                    end_counter += 1
                if start_counter != 0 and start_counter == end_counter:
                    self.deleteStatements(file_name, idx_to_del) 
                    return idx_to_del[0]
            mutation_line_number += 1


class AndOr(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        self._replace_strategy = {
            'regex': r'\s&&\s',
            'replaceString': ' || '
        }


class IfTrue(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        self._replace_strategy = {
            'regex': r'(?<=if\s)(.*)(?=\s\{)',
            'replaceString': 'true'
        }


class IfFalse(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        self._replace_strategy = {
            'regex': r'(?<=if\s)(.*)(?=\s\{)',
            'replaceString': 'false'
        }


class ModifyComparision(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        self._replace_strategy = {
            'regex': r"(?<=\s\<)\=\s|(?<=\s\>)\=\s",
            'replaceString': ' '
        }


class MinusToPlus(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        self._replace_strategy = {
            'regex': r'(?<=\s)\-(?=\s.+)|(?<=\s)\-(?=\=)',
            'replaceString': '+'
        }


class PlusToMinus(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        self._replace_strategy = {
            'regex': r"(?<=[^\"]\s)\+(?=\s[^A-Z'?\":\{]+)|(?<=\s)\+(?=\=)",
            'replaceString': '-'
        }


class DuplicateLine(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        self._strategy_name = "duplicate"
        self._replace_strategy = {
            'regex': r'.*?append\(.*?\).*?;',
            'replaceString': None,
        }


class DeleteStatement(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        self._strategy_name = "delete_if_statement"
        self._delete_strategy = {
            'ifBlock': r'[^a-z]\sif(.+){',
            'elseBlock': r'\selse(.+){'
        }


def get_strategies():
    return AndOr, IfTrue, IfFalse, ModifyComparision, PlusToMinus, MinusToPlus, DuplicateLine, DeleteStatement


class Mutator:
    def __init__(self, strategy):
        self._strategy = strategy

    def mutate(self, file_name):
        return self._strategy.mutate(file_name)
