# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

SIZE = 8

with open("../benchmarks/many-images.yaml", "w") as text_file:
    text_file.write("root:\n")
    text_file.write("  items:\n")
    for y in range(0, 64):
        yb = SIZE * y
        for x in range(0, 128):
            xb = SIZE * x
            text_file.write("    - image: solid-color({0}, {1}, 0, 255, {2}, {2})\n".format(x, y, SIZE))
            text_file.write("      bounds: {0} {1} {2} {2}\n".format(xb, yb, SIZE))
