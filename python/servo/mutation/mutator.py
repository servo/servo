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
import logging


def is_comment(line):
    return re.search(r'\/\/.*', line)


class Strategy:
    def __init__(self):
        self._replace_strategy = {}
        self._strategy_name = ""
        logging.basicConfig(format="%(asctime)s %(levelname)s: %(message)s", level=logging.DEBUG)

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
                        self.line = re.sub(self._replace_strategy['regex'], self._replace_strategy['replaceString'], line)
                    else:
                        if self._strategy_name == "duplicate":
                            replacement = line + "\n" + line
                            self.line = re.sub(self._replace_strategy['regex'], replacement, line)

                logging.info(line.rstrip())
            return mutation_line_number


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


def get_strategies():
    return AndOr, IfTrue, IfFalse, MinusToPlus, DuplicateLine


class Mutator:
    def __init__(self, strategy):
        self._strategy = strategy

    def mutate(self, file_name):
        return self._strategy.mutate_random_line(file_name)
