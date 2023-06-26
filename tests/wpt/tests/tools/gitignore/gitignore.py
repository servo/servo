import re
import os
import itertools
from collections import defaultdict
from typing import (Any, Dict, Iterable, List, MutableMapping, Optional, Pattern, Tuple, TypeVar,
                    Union, cast)


T = TypeVar('T')

end_space = re.compile(r"([^\\]\s)*$")


def fnmatch_translate(pat: bytes) -> Tuple[bool, Pattern[bytes]]:
    parts = []
    seq: Optional[int] = None
    i = 0
    any_char = b"[^/]"
    if pat[0:1] == b"/":
        parts.append(b"^")
        pat = pat[1:]
    else:
        # By default match the entire path up to a /
        # but if / doesn't appear in the pattern we will mark is as
        # a name pattern and just produce a pattern that matches against
        # the filename
        parts.append(b"^(?:.*/)?")

    name_pattern = True
    if pat[-1:] == b"/":
        # If the last character is / match this directory or any subdirectory
        pat = pat[:-1]
        suffix = b"(?:/|$)"
    else:
        suffix = b"$"
    while i < len(pat):
        c = pat[i:i+1]
        if c == b"\\":
            if i < len(pat) - 1:
                i += 1
                c = pat[i:i+1]
                parts.append(re.escape(c))
            else:
                raise ValueError
        elif seq is not None:
            # TODO: this doesn't really handle invalid sequences in the right way
            if c == b"]":
                seq = None
                if parts[-1] == b"[":
                    parts = parts[:-1]
                elif parts[-1] == b"^" and parts[-2] == b"[":
                    raise ValueError
                else:
                    parts.append(c)
            elif c == b"-":
                parts.append(c)
            elif c == b"[":
                raise ValueError
            else:
                parts.append(re.escape(c))
        elif c == b"[":
            parts.append(b"[")
            if i < len(pat) - 1 and pat[i+1:i+2] in (b"!", b"^"):
                parts.append(b"^")
                i += 1
            seq = i
        elif c == b"*":
            if i < len(pat) - 1 and pat[i+1:i+2] == b"*":
                if i > 0 and pat[i-1:i] != b"/":
                    raise ValueError
                parts.append(b".*")
                i += 1
                if i < len(pat) - 1 and pat[i+1:i+2] != b"/":
                    raise ValueError
            else:
                parts.append(any_char + b"*")
        elif c == b"?":
            parts.append(any_char)
        elif c == b"/" and not seq:
            name_pattern = False
            parts.append(c)
        else:
            parts.append(re.escape(c))
        i += 1

    if name_pattern:
        parts[0] = b"^"

    if seq is not None:
        raise ValueError
    parts.append(suffix)
    try:
        return name_pattern, re.compile(b"".join(parts))
    except Exception:
        raise ValueError

# Regexp matching rules that have to be converted to patterns
pattern_re = re.compile(br".*[\*\[\?]")


def parse_line(line: bytes) -> Optional[Tuple[bool, bool, bool, Union[Tuple[bytes, ...], Tuple[bool, Pattern[bytes]]]]]:
    line = line.rstrip()
    if not line or line[0:1] == b"#":
        return None

    invert = line[0:1] == b"!"
    if invert:
        line = line[1:]

    dir_only = line[-1:] == b"/"

    if dir_only:
        line = line[:-1]

    # Could make a special case for **/foo, but we don't have any patterns like that
    if not invert and not pattern_re.match(line):
        literal = True
        pattern: Union[Tuple[bytes, ...], Tuple[bool, Pattern[bytes]]] = tuple(line.rsplit(b"/", 1))
    else:
        pattern = fnmatch_translate(line)
        literal = False

    return invert, dir_only, literal, pattern


class PathFilter:
    def __init__(self, root: bytes, extras: Optional[List[bytes]] = None, cache: Optional[MutableMapping[bytes, bool]] = None) -> None:
        if root:
            ignore_path: Optional[bytes] = os.path.join(root, b".gitignore")
        else:
            ignore_path = None
        if not ignore_path and not extras:
            self.trivial = True
            return
        self.trivial = False

        self.literals_file: Dict[Optional[bytes], Dict[bytes, List[Tuple[bool, Pattern[bytes]]]]] = defaultdict(dict)
        self.literals_dir: Dict[Optional[bytes], Dict[bytes, List[Tuple[bool, Pattern[bytes]]]]] = defaultdict(dict)
        self.patterns_file: List[Tuple[Tuple[bool, Pattern[bytes]], List[Tuple[bool, Pattern[bytes]]]]] = []
        self.patterns_dir: List[Tuple[Tuple[bool, Pattern[bytes]], List[Tuple[bool, Pattern[bytes]]]]] = []

        if cache is None:
            cache = {}
        self.cache: MutableMapping[bytes, bool] = cache

        if extras is None:
            extras = []

        if ignore_path and os.path.exists(ignore_path):
            args: Tuple[Optional[bytes], List[bytes]] = (ignore_path, extras)
        else:
            args = None, extras
        self._read_ignore(*args)

    def _read_ignore(self, ignore_path: Optional[bytes], extras: List[bytes]) -> None:
        if ignore_path is not None:
            with open(ignore_path, "rb") as f:
                for line in f:
                    self._read_line(line)
        for line in extras:
            self._read_line(line)

    def _read_line(self, line: bytes) -> None:
        parsed = parse_line(line)
        if not parsed:
            return
        invert, dir_only, literal, rule = parsed

        if invert:
            # For exclude rules, we attach the rules to all preceeding patterns, so
            # that we can match patterns out of order and check if they were later
            # overriden by an exclude rule
            assert not literal
            rule = cast(Tuple[bool, Pattern[bytes]], rule)
            if not dir_only:
                rules_iter: Iterable[Tuple[Any, List[Tuple[bool, Pattern[bytes]]]]] = itertools.chain(
                    itertools.chain(*(item.items() for item in self.literals_dir.values())),
                    itertools.chain(*(item.items() for item in self.literals_file.values())),
                    self.patterns_dir,
                    self.patterns_file)
            else:
                rules_iter = itertools.chain(
                    itertools.chain(*(item.items() for item in self.literals_dir.values())),
                    self.patterns_dir)

            for rules in rules_iter:
                rules[1].append(rule)
        else:
            if literal:
                rule = cast(Tuple[bytes, ...], rule)
                if len(rule) == 1:
                    dir_name, pattern = None, rule[0]  # type: Tuple[Optional[bytes], bytes]
                else:
                    dir_name, pattern = rule
                self.literals_dir[dir_name][pattern] = []
                if not dir_only:
                    self.literals_file[dir_name][pattern] = []
            else:
                rule = cast(Tuple[bool, Pattern[bytes]], rule)
                self.patterns_dir.append((rule, []))
                if not dir_only:
                    self.patterns_file.append((rule, []))

    def filter(self,
               iterator: Iterable[Tuple[bytes, List[Tuple[bytes, T]], List[Tuple[bytes, T]]]]
               ) -> Iterable[Tuple[bytes, List[Tuple[bytes, T]], List[Tuple[bytes, T]]]]:
        empty: Dict[Any, Any] = {}
        for dirpath, dirnames, filenames in iterator:
            orig_dirpath = dirpath
            path_sep = os.path.sep.encode()
            if path_sep != b"/":
                dirpath = dirpath.replace(path_sep, b"/")

            keep_dirs: List[Tuple[bytes, T]] = []
            keep_files: List[Tuple[bytes, T]] = []

            for iter_items, literals, patterns, target, suffix in [
                    (dirnames, self.literals_dir, self.patterns_dir, keep_dirs, b"/"),
                    (filenames, self.literals_file, self.patterns_file, keep_files, b"")]:
                for item in iter_items:
                    name = item[0]
                    if dirpath:
                        path = b"%s/%s" % (dirpath, name) + suffix
                    else:
                        path = name + suffix
                    if path in self.cache:
                        if not self.cache[path]:
                            target.append(item)
                        continue
                    for rule_dir in [None, dirpath if dirpath != b"." else b""]:
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
            assert not any(b".git" == name for name, _ in dirnames)
            yield orig_dirpath, dirnames, keep_files

    def __call__(self,
                 iterator: Iterable[Tuple[bytes, List[Tuple[bytes, T]], List[Tuple[bytes, T]]]]
                 ) -> Iterable[Tuple[bytes, List[Tuple[bytes, T]], List[Tuple[bytes, T]]]]:
        if self.trivial:
            return iterator

        return self.filter(iterator)


def has_ignore(dirpath: bytes) -> bool:
    return os.path.exists(os.path.join(dirpath, b".gitignore"))
