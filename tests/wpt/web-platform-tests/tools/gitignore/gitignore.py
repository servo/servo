import re
import os
import itertools
from six import itervalues, iteritems
from collections import defaultdict

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Any
    from typing import Dict
    from typing import Iterable
    from typing import List
    from typing import MutableMapping
    from typing import Optional
    from typing import Pattern
    from typing import Tuple
    from typing import TypeVar
    from typing import Union
    from typing import cast

    T = TypeVar('T')


end_space = re.compile(r"([^\\]\s)*$")


def fnmatch_translate(pat):
    # type: (str) -> Tuple[bool, Pattern[str]]
    parts = []
    seq = None
    i = 0
    any_char = "[^/]"
    if pat[0] == "/":
        parts.append("^")
        pat = pat[1:]
    else:
        # By default match the entire path up to a /
        # but if / doesn't appear in the pattern we will mark is as
        # a name pattern and just produce a pattern that matches against
        # the filename
        parts.append("^(?:.*/)?")

    name_pattern = True
    if pat[-1] == "/":
        # If the last character is / match this directory or any subdirectory
        pat = pat[:-1]
        suffix = "(?:/|$)"
    else:
        suffix = "$"
    while i < len(pat):
        c = pat[i]
        if c == "\\":
            if i < len(pat) - 1:
                i += 1
                c = pat[i]
                parts.append(re.escape(c))
            else:
                raise ValueError
        elif seq is not None:
            # TODO: this doesn't really handle invalid sequences in the right way
            if c == "]":
                seq = None
                if parts[-1] == "[":
                    parts = parts[:-1]
                elif parts[-1] == "^" and parts[-2] == "[":
                    parts = parts[:-2]
                else:
                    parts.append(c)
            elif c == "-":
                parts.append(c)
            else:
                parts += re.escape(c)
        elif c == "[":
            parts.append("[")
            if i < len(pat) - 1 and pat[i+1] in ("!", "^"):
                parts.append("^")
                i += 1
            seq = i
        elif c == "*":
            if i < len(pat) - 1 and pat[i+1] == "*":
                if i > 0 and pat[i-1] != "/":
                    raise ValueError
                parts.append(".*")
                i += 1
                if i < len(pat) - 1 and pat[i+1] != "/":
                    raise ValueError
            else:
                parts.append(any_char + "*")
        elif c == "?":
            parts.append(any_char)
        elif c == "/" and not seq:
            name_pattern = False
            parts.append(c)
        else:
            parts.append(re.escape(c))
        i += 1

    if name_pattern:
        parts[0] = "^"

    if seq is not None:
        raise ValueError
    parts.append(suffix)
    try:
        return name_pattern, re.compile("".join(parts))
    except Exception:
        raise ValueError

# Regexp matching rules that have to be converted to patterns
pattern_re = re.compile(r".*[\*\[\?]")


def parse_line(line):
    # type: (str) -> Optional[Tuple[bool, bool, bool, Union[Tuple[str, ...], Tuple[bool, Pattern[str]]]]]
    line = line.rstrip()
    if not line or line[0] == "#":
        return None

    invert = line[0] == "!"
    if invert:
        line = line[1:]

    dir_only = line[-1] == "/"

    if dir_only:
        line = line[:-1]

    # Could make a special case for **/foo, but we don't have any patterns like that
    if not invert and not pattern_re.match(line):
        literal = True
        pattern = tuple(line.rsplit("/", 1))  # type: Union[Tuple[str, ...], Tuple[bool, Pattern[str]]]
    else:
        pattern = fnmatch_translate(line)
        literal = False

    return invert, dir_only, literal, pattern


class PathFilter(object):
    def __init__(self, root, extras=None, cache=None):
        # type: (str, Optional[List[str]], Optional[MutableMapping[str, bool]]) -> None
        if root:
            ignore_path = os.path.join(root, ".gitignore")  # type: Optional[str]
        else:
            ignore_path = None
        if not ignore_path and not extras:
            self.trivial = True
            return
        self.trivial = False

        self.literals_file = defaultdict(dict)  # type: Dict[Optional[str], Dict[str, List[Tuple[bool, Pattern[str]]]]]
        self.literals_dir = defaultdict(dict)  # type: Dict[Optional[str], Dict[str, List[Tuple[bool, Pattern[str]]]]]
        self.patterns_file = []  # type: List[Tuple[Tuple[bool, Pattern[str]], List[Tuple[bool, Pattern[str]]]]]
        self.patterns_dir = []  # type: List[Tuple[Tuple[bool, Pattern[str]], List[Tuple[bool, Pattern[str]]]]]
        self.cache = cache or {}  # type: MutableMapping[str, bool]

        if extras is None:
            extras = []

        if ignore_path and os.path.exists(ignore_path):
            args = ignore_path, extras  # type: Tuple[Optional[str], List[str]]
        else:
            args = None, extras
        self._read_ignore(*args)

    def _read_ignore(self, ignore_path, extras):
        # type: (Optional[str], List[str]) -> None
        if ignore_path is not None:
            with open(ignore_path) as f:
                for line in f:
                    self._read_line(line)
        for line in extras:
            self._read_line(line)

    def _read_line(self, line):
        # type: (str) -> None
        parsed = parse_line(line)
        if not parsed:
            return
        invert, dir_only, literal, rule = parsed

        if invert:
            # For exclude rules, we attach the rules to all preceeding patterns, so
            # that we can match patterns out of order and check if they were later
            # overriden by an exclude rule
            assert not literal
            if MYPY:
                rule = cast(Tuple[bool, Pattern[str]], rule)
            if not dir_only:
                rules_iter = itertools.chain(
                    itertools.chain(*(iteritems(item) for item in itervalues(self.literals_dir))),
                    itertools.chain(*(iteritems(item) for item in itervalues(self.literals_file))),
                    self.patterns_dir,
                    self.patterns_file)  # type: Iterable[Tuple[Any, List[Tuple[bool, Pattern[str]]]]]
            else:
                rules_iter = itertools.chain(
                    itertools.chain(*(iteritems(item) for item in itervalues(self.literals_dir))),
                    self.patterns_dir)

            for rules in rules_iter:
                rules[1].append(rule)
        else:
            if literal:
                if MYPY:
                    rule = cast(Tuple[str, ...], rule)
                if len(rule) == 1:
                    dir_name, pattern = None, rule[0]  # type: Tuple[Optional[str], str]
                else:
                    dir_name, pattern = rule
                self.literals_dir[dir_name][pattern] = []
                if not dir_only:
                    self.literals_file[dir_name][pattern] = []
            else:
                if MYPY:
                    rule = cast(Tuple[bool, Pattern[str]], rule)
                self.patterns_dir.append((rule, []))
                if not dir_only:
                    self.patterns_file.append((rule, []))

    def filter(self,
               iterator  # type: Iterable[Tuple[str, List[Tuple[str, T]], List[Tuple[str, T]]]]
               ):
        # type: (...) -> Iterable[Tuple[str, List[Tuple[str, T]], List[Tuple[str, T]]]]
        empty = {}  # type: Dict[Any, Any]
        for dirpath, dirnames, filenames in iterator:
            orig_dirpath = dirpath
            if os.path.sep != "/":
                dirpath = dirpath.replace(os.path.sep, "/")

            keep_dirs = []  # type: List[Tuple[str, T]]
            keep_files = []  # type: List[Tuple[str, T]]

            for iter_items, literals, patterns, target, suffix in [
                    (dirnames, self.literals_dir, self.patterns_dir, keep_dirs, "/"),
                    (filenames, self.literals_file, self.patterns_file, keep_files, "")]:
                for item in iter_items:
                    name = item[0]
                    if dirpath:
                        path = "%s/%s" % (dirpath, name) + suffix
                    else:
                        path = name + suffix
                    if path in self.cache:
                        if not self.cache[path]:
                            target.append(item)
                        continue
                    for rule_dir in [None, dirpath]:
                        if name in literals.get(rule_dir, empty):
                            exclude = literals[rule_dir][name]
                            if not any(rule.match(name if name_only else path)
                                       for name_only, rule in exclude):
                                # Skip this item
                                self.cache[path] = True
                                break
                    else:
                        for (component_only, pattern), exclude in patterns:
                            if component_only:
                                match = pattern.match(name)
                            else:
                                match = pattern.match(path)
                            if match:
                                if not any(rule.match(name if name_only else path)
                                           for name_only, rule in exclude):
                                    # Skip this item
                                    self.cache[path] = True
                                    break
                        else:
                            self.cache[path] = False
                            target.append(item)

            dirnames[:] = keep_dirs
            assert not any(".git" == name for name, _ in dirnames)
            yield orig_dirpath, dirnames, keep_files

    def __call__(self,
                 iterator  # type: Iterable[Tuple[str, List[Tuple[str, T]], List[Tuple[str, T]]]]
                 ):
        # type: (...) -> Iterable[Tuple[str, List[Tuple[str, T]], List[Tuple[str, T]]]]
        if self.trivial:
            return iterator

        return self.filter(iterator)


def has_ignore(dirpath):
    # type: (str) -> bool
    return os.path.exists(os.path.join(dirpath, ".gitignore"))
