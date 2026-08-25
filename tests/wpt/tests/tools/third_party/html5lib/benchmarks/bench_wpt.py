import io
import os
import sys

import pyperf

sys.path[0:0] = [os.path.join(os.path.dirname(__file__), "..")]
import html5lib  # noqa: E402


def bench_html5lib(fh):
    fh.seek(0)
    html5lib.parse(fh, treebuilder="etree", useChardet=False)


def add_cmdline_args(cmd, args):
    if args.benchmark:
        cmd.append(args.benchmark)


BENCHMARKS = {}
for root, dirs, files in os.walk(os.path.join(os.path.dirname(os.path.abspath(__file__)), "data", "wpt")):
    for f in files:
        if f.endswith(".html"):
            BENCHMARKS[f[: -len(".html")]] = os.path.join(root, f)


if __name__ == "__main__":
    runner = pyperf.Runner(add_cmdline_args=add_cmdline_args)
    runner.metadata["description"] = "Run parser benchmarks from WPT"
    runner.argparser.add_argument("benchmark", nargs="?", choices=sorted(BENCHMARKS))

    args = runner.parse_args()
    if args.benchmark:
        benchmarks = (args.benchmark,)
    else:
        benchmarks = sorted(BENCHMARKS)

    for bench in benchmarks:
        name = "wpt_%s" % bench
        path = BENCHMARKS[bench]
        with open(path, "rb") as fh:
            fh2 = io.BytesIO(fh.read())

        runner.bench_func(name, bench_html5lib, fh2)
