#!/usr/bin/env python3
# Requires Python 3.7+ — earlier versions may produce operators/operands in
# random order in the JSON output due to undefined dictionary iteration order.

import json
import re
import sys
from pathlib import Path

import requests
from bs4 import BeautifulSoup


INDEX_BS_URL = (
    'https://raw.githubusercontent.com/webmachinelearning/webnn/'
    'refs/heads/main/index.bs'
)
OUTPUT_PATH = (
    Path(__file__).resolve().parent.parent
    / 'resources' / 'required_datatypes_ranks.json'
)

# Precompiled regexes (module-level to avoid recompiling per call/row).
_DIGITS_RE = re.compile(r'\d+')
_OUTPUT_INDEX_RE = re.compile(r'\d+')
_OPERAND_REF_RE = re.compile(r'\{\{(.*?)\}\}')
_OPERATOR_RE = re.compile(r'MLGraphBuilder/(\w+)\(.*?\)')
_DATATYPE_RE = re.compile(r'MLOperandDataType/"([^"]+)"')


def _get_min_max_ranks(rank_string):
    ranks = _DIGITS_RE.findall(rank_string)
    if not ranks:
        return {}
    minimum_rank = int(ranks[0])
    maximum_rank = int(ranks[1]) if len(ranks) > 1 else minimum_rank
    return {'min': minimum_rank, 'max': maximum_rank}


def _get_operand_name(operand_string):
    if 'output' in operand_string:
        if 'outputs' in operand_string:
            index_match = _OUTPUT_INDEX_RE.search(operand_string)
            return f'output{index_match.group(0)}' if index_match else 'outputs'
        return 'output'

    match = _OPERAND_REF_RE.search(operand_string)
    if not match:
        return ''
    name = match.group(1)
    # Strip any namespace prefix like "Foo/bar" -> "bar".
    return name.rsplit('/', 1)[-1]


def generate():
    print(f'Fetching WebNN spec from:\n  {INDEX_BS_URL}')
    try:
        response = requests.get(INDEX_BS_URL, timeout=30)
        response.raise_for_status()
    except requests.RequestException as err:
        print(f'  [error] failed to fetch index.bs: {err}', file=sys.stderr)
        return 1
    print(f'  [ok] downloaded {len(response.text):,} characters')

    print('Parsing tensor-limits tables...')
    soup = BeautifulSoup(response.text, 'html.parser')
    results = {}

    tables = soup.find_all(
        'table', id=lambda x: x and x.startswith('tensor-limits-'))
    print(f'  [ok] found {len(tables)} tensor-limits table(s)')

    total_rows = 0
    for table in tables:
        operators = _OPERATOR_RE.findall(table.get('link-for', ''))
        if not operators:
            continue

        for row in table.find_all('tr'):
            cells = row.find_all('td')
            if not cells:
                continue  # Skip header rows.

            operand_name = _get_operand_name(cells[0].get_text(strip=True))
            data_types = _DATATYPE_RE.findall(
                cells[2].get_text(strip=True))
            rank_range = _get_min_max_ranks(cells[4].get_text(strip=True))

            entry = {'dataTypes': data_types, 'rankRange': rank_range}
            for operator in operators:
                results.setdefault(operator, {})[operand_name] = entry
            total_rows += 1

    print(f'  [ok] extracted {total_rows} operand row(s) for '
          f'{len(results)} operator(s)')

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    # Use newline='\n' to avoid Windows CRLF translation, which would
    # otherwise produce CR-at-EOL lint errors in the generated JSON.
    with OUTPUT_PATH.open('w', encoding='utf-8', newline='\n') as json_file:
        json_file.write(_format_results(results))
        json_file.write('\n')
    print(f'Wrote results to:\n  {OUTPUT_PATH}')
    print('Done.')
    return 0


def _format_results(results, indent='  '):
    """Serialize `results` to JSON where each operand's `dataTypes` array and
    `rankRange` object are kept on a single line, while the rest of the
    structure is pretty-printed with the given indent.

    Example output for one operand entry:
        "input": {
          "dataTypes": ["float32", "float16", "int32"],
          "rankRange": {"min": 1, "max": 5}
        }
    """
    lines = ['{']
    operators = list(results.items())
    for op_idx, (op_name, operands) in enumerate(operators):
        op_trailing = ',' if op_idx < len(operators) - 1 else ''
        lines.append(f'{indent}{json.dumps(op_name)}: {{')

        operand_items = list(operands.items())
        for od_idx, (operand_name, entry) in enumerate(operand_items):
            od_trailing = ',' if od_idx < len(operand_items) - 1 else ''
            lines.append(f'{indent * 2}{json.dumps(operand_name)}: {{')
            lines.append(
                f'{indent * 3}"dataTypes": '
                f'{json.dumps(entry.get("dataTypes", []))},')
            lines.append(
                f'{indent * 3}"rankRange": '
                f'{json.dumps(entry.get("rankRange", {}))}')
            lines.append(f'{indent * 2}}}{od_trailing}')

        lines.append(f'{indent}}}{op_trailing}')
    lines.append('}')
    return '\n'.join(lines)


if __name__ == '__main__':
    if sys.version_info < (3, 7):
        print('ERROR: This script requires Python 3.7+', file=sys.stderr)
        print(f'  Current version: {sys.version}', file=sys.stderr)
        sys.exit(1)

    sys.exit(generate())