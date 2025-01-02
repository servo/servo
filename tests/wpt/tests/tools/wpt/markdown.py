# mypy: allow-untyped-defs

from functools import reduce

def format_comment_title(product):
    """Produce a Markdown-formatted string based on a given "product"--a string
    containing a browser identifier optionally followed by a colon and a
    release channel. (For example: "firefox" or "chrome:dev".) The generated
    title string is used both to create new comments and to locate (and
    subsequently update) previously-submitted comments."""
    parts = product.split(":")
    title = parts[0].title()

    if len(parts) > 1:
        title += " (%s)" % parts[1]

    return "# %s #" % title


def markdown_adjust(s):
    """Escape problematic markdown sequences."""
    s = s.replace('\t', '\\t')
    s = s.replace('\n', '\\n')
    s = s.replace('\r', '\\r')
    s = s.replace('`', '')
    s = s.replace('|', '\\|')
    return s


def table(headings, data, log):
    """Create and log data to specified logger in tabular format."""
    cols = range(len(headings))
    assert all(len(item) == len(cols) for item in data)
    max_widths = reduce(lambda prev, cur: [(len(cur[i]) + 2)
                                           if (len(cur[i]) + 2) > prev[i]
                                           else prev[i]
                                           for i in cols],
                        data,
                        [len(item) + 2 for item in headings])
    log("|%s|" % "|".join(item.center(max_widths[i]) for i, item in enumerate(headings)))
    log("|%s|" % "|".join("-" * max_widths[i] for i in cols))
    for row in data:
        log("|%s|" % "|".join(" %s" % row[i].ljust(max_widths[i] - 1) for i in cols))
    log("")
