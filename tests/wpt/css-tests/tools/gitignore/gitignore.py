import itertools
import re
import os

end_space = re.compile(r"([^\\]\s)*$")


def fnmatch_translate(pat, path_name=False):
    parts = []
    seq = False
    i = 0
    if pat[0] == "/" or path_name:
        parts.append("^")
        any_char = "[^/]"
        if pat[0] == "/":
            pat = pat[1:]
    else:
        any_char = "."
        parts.append("^(?:.*/)?")
    while i < len(pat):
        c = pat[i]
        if c == "\\":
            if i < len(pat) - 1:
                i += 1
                c = pat[i]
                parts.append(re.escape(c))
            else:
                raise ValueError
        elif seq:
            if c == "]":
                seq = False
                # First two cases are to deal with the case where / is the only character
                # in the sequence but path_name is True so it shouldn't match anything
                if parts[-1] == "[":
                    parts = parts[:-1]
                elif parts[-1] == "^" and parts[-2] == "[":
                    parts = parts[:-2]
                else:
                    parts.append(c)
            elif c == "-":
                parts.append(c)
            elif not (path_name and c == "/"):
                parts += re.escape(c)
        elif c == "[":
            parts.append("[")
            if i < len(pat) - 1 and pat[i+1] in ("!", "^"):
                parts.append("^")
                i += 1
            seq = True
        elif c == "*":
            if i < len(pat) - 1 and pat[i+1] == "*":
                parts.append(any_char + "*")
                i += 1
                if i < len(pat) - 1 and pat[i+1] == "*":
                    raise ValueError
            else:
                parts.append(any_char + "*")
        elif c == "?":
            parts.append(any_char)
        else:
            parts.append(re.escape(c))
        i += 1

    if seq:
        raise ValueError
    parts.append("$")
    try:
        return re.compile("".join(parts))
    except:
        raise


def parse_line(line):
    line = line.rstrip()
    if not line or line[0] == "#":
        return

    invert = line[0] == "!"
    if invert:
        line = line[1:]

    dir_only = line[-1] == "/"

    if dir_only:
        line = line[:-1]

    return invert, dir_only, fnmatch_translate(line, "/" in line)


class PathFilter(object):
    def __init__(self, root, extras=None):
        if root:
            ignore_path = os.path.join(root, ".gitignore")
        else:
            ignore_path = None
        if not ignore_path and not extras:
            self.trivial = True
            return
        self.trivial = False

        self.rules_file = []
        self.rules_dir = []

        if extras is None:
            extras = []

        if ignore_path and os.path.exists(ignore_path):
            self._read_ignore(ignore_path)

        for item in extras:
            self._read_line(item)

    def _read_ignore(self, ignore_path):
        with open(ignore_path) as f:
            for line in f:
                self._read_line(line)

    def _read_line(self, line):
        parsed = parse_line(line)
        if not parsed:
            return
        invert, dir_only, regexp = parsed
        if dir_only:
            self.rules_dir.append((regexp, invert))
        else:
            self.rules_file.append((regexp, invert))

    def __call__(self, path):
        if os.path.sep != "/":
            path = path.replace(os.path.sep, "/")

        if self.trivial:
            return True

        path_is_dir = path[-1] == "/"
        if path_is_dir:
            path = path[:-1]
            rules = self.rules_dir
        else:
            rules = self.rules_file

        include = True
        for regexp, invert in rules:
            if not include and invert and regexp.match(path):
                include = True
            elif include and not invert and regexp.match(path):
                include = False
        return include
