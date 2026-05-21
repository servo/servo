import re

def insert_symbol(symbol, name):
    base = """<first>HT<in-http>TP<before-slash>/<after-slash>1<before-dot>.<after-dot>1<after-ver> <before-num>200<after-num> <before-reason>OK<after-reason>\r
<leading>X-Frame<in-name>-Options<before-colon>:<after-colon>DE<in-value>NY<after-value>\r
Content-Length: 5\r
\r
Test."""
    for pos in ["first", "in-http", "before-slash", "after-slash", "before-dot", "after-dot", "after-ver", "before-num", "after-num", "before-reason", "after-reason", "leading", "in-name", "before-colon", "after-colon", "in-value", "after-value"]:
        final = base.replace(f"<{pos}>", symbol)
        # Remove any remaining placeholders in angle brackets
        final = re.sub(r"<[^>]+>", "", final)
        with open(f"../resources/parsing_{name}_{pos}.asis", "w") as f:
            f.write(final)


for symbol, name in [("\n", "LF"), ("\r", "CR"), ("\t", "HTAB"), (" ", "SP"), ("\x00", "NUL"), (":", "COLON")]:
    insert_symbol(symbol, name)