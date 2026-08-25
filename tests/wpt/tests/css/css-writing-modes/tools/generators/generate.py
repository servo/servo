#!/usr/bin/env python3
import os
import string

from typing import List, Tuple

test_template = """<h3>{number}: {title}</h3>
<div class="test">
    {html}
</div>"""


def generate_test_list() -> List[Tuple[str, str]]:
    test_list = [];
    outers = [
        ["inline-block", '<div class="inline-block">', '</div><span class="next">ZZ</span>'],
        ["float", '<div class="float">', '</div><span class="next">ZZ</span>'],
        ["table-cell", '<table><tr><td>', '</td><td class="next">ZZ</td></tr></table>']];
    middles = [
        None,
        ["inline-block", '<div class="inline-block">', '</div>']];
    targets = [
        ["block", '<div class="target">HH</div>'],
        ["inline", '<span class="target">HH</span>'],
        ["block with borders", '<div class="target border">HHH</div>'],
        ["inline with borders", '<span class="target border">HHH</span>']];
    for outer in outers:
        for middle in middles:
            for target in targets:
                title = target[0];
                html = target[1];
                if middle is not None:
                    title += " in " + middle[0];
                    html = middle[1] + html + middle[2];
                title = "Shrink-to-fit " + outer[0] + " with a child of orthogonal " + title;
                html = outer[1] + html + outer[2];
                test_list.append((title, html));
    return test_list


def read_template() -> str:
    with open("template.html") as f:
        return f.read()


def main():
    template = read_template()
    test_list = generate_test_list()

    dest_dir = os.path.abspath(
        os.path.join(os.path.dirname(os.path.abspath(__file__)),
                     os.path.pardir,
                     os.path.pardir))

    for index in range(-1, len(test_list)):
        if index == -1:
            offset = 0
            suffix = ""
            tests = test_list
            title = "Shrink-to-fit with orthogonal children"
            flags = " combo"
        else:
            offset = index
            suffix = string.ascii_letters[index]
            tests = [test_list[index]]
            title = tests[0][0]
            flags = ""

        filename = f"orthogonal-parent-shrink-to-fit-001{suffix}.html"

        tests_data = []
        for idx, (test_title, html) in enumerate(tests):
            number = offset + idx + 1
            tests_data.append(test_template.format(number=number,
                                                   title=test_title,
                                                   html=html))

        output = template.replace("{{title}}", title)
        output = output.replace("{{flags}}", flags)
        output = output.replace("{{tests}}", "\n".join(tests_data))
        with open(os.path.join(dest_dir, filename), "w") as f:
            f.write(output)


if __name__ == "__main__":
    main()
