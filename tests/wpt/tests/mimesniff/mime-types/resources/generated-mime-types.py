import json
import os

here = os.path.dirname(__file__)

def isHTTPTokenCodePoint(cp):
    if cp in (0x21, 0x23, 0x24, 0x25, 0x26, 0x27, 0x2A, 0x2B, 0x2D, 0x2E, 0x5E, 0x5F, 0x60, 0x7C, 0x7E) or (cp >= 0x30 and cp <= 0x39) or (cp >= 0x41 and cp <= 0x5A) or (cp >= 0x61 and cp <= 0x7A):
        return True
    else:
        return False

def isHTTPQuotedStringTokenCodePoint(cp):
    if cp == 0x09 or (cp >= 0x20 and cp <= 0x7E) or (cp >= 0x80 and cp <= 0xFF):
        return True
    else:
        return False

tests = []

for cp in range(0x00, 0x100):
    if isHTTPTokenCodePoint(cp):
        continue
    for scenario in ("type", "subtype", "name", "value"):
        if scenario == "type" or scenario == "subtype":
            if cp == 0x2F: # /
                continue
            if scenario == "type":
                test = chr(cp) + "/x"
            else:
                test = "x/" + chr(cp)
            tests.append({"input": test, "output": None})
        elif scenario == "name":
            if cp == 0x3B or cp == 0x3D: # ; =
                continue
            tests.append({"input": "x/x;" + chr(cp) + "=x;bonus=x", "output": "x/x;bonus=x"})
        elif scenario == "value":
            if cp == 0x09 or cp == 0x20 or cp == 0x22 or cp == 0x3B or cp == 0x5C: # TAB SP " ; \
                continue
            if isHTTPQuotedStringTokenCodePoint(cp):
                testOutput = "x/x;x=\"" + chr(cp) + "\";bonus=x"
            else:
                testOutput = "x/x;bonus=x"
            tests.append({"input": "x/x;x=" + chr(cp) + ";bonus=x", "output": testOutput})
            tests.append({"input": "x/x;x=\"" + chr(cp) + "\";bonus=x", "output": testOutput})

handle = open(os.path.join(here, "generated-mime-types.json"), "w")
handle.write(json.dumps(tests, indent=2, separators=(',', ': ')))
handle.write("\n")
