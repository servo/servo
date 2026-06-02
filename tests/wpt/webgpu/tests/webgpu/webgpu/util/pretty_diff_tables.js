/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { range } from '../../common/util/util.js'; /**
 * @returns a function that converts numerics to strings, depending on if they
 * should be treated as integers or not.
 */
export function numericToStringBuilder(is_integer) {
  if (is_integer) {
    return (val) => {
      if (typeof val === 'number') {
        return val.toFixed();
      }
      return val.toString();
    };
  }

  return (val) => {
    if (typeof val === 'number') {
      return val.toPrecision(6);
    }
    return val.toString();
  };
}

/**
 * Pretty-prints a "table" of cell values (each being `number | string`), right-aligned.
 * Each row may be any iterator, including lazily-generated (potentially infinite) rows.
 *
 * The first argument is the printing options:
 *  - fillToWidth: Keep printing columns (as long as there is data) until this width is passed.
 *    If there is more data, "..." is appended.
 *  - numberToString: if a cell value is a number, this is used to stringify it.
 *
 * Each remaining argument provides one row for the table.
 */
export function generatePrettyTable(
{
  fillToWidth,
  numericToString
},
rows)
{
  const rowStrings = range(rows.length, () => '');
  let totalTableWidth = 0;
  const iters = rows.map((row) => row[Symbol.iterator]());

  // Loop over columns
  for (;;) {
    const cellsForColumn = iters.map((iter) => {
      const r = iter.next(); // Advance the iterator for each row, in lock-step.
      if (r.done) {
        return undefined;
      }
      if (typeof r.value === 'number' || typeof r.value === 'bigint') {
        return numericToString(r.value);
      }
      return r.value;
    });
    if (cellsForColumn.every((cell) => cell === undefined)) break;

    // Maximum width of any cell in this column, plus one for space between columns
    // (also inserts a space at the left of the first column).
    const colWidth = Math.max(...cellsForColumn.map((c) => c === undefined ? 0 : c.length)) + 1;
    for (let row = 0; row < rowStrings.length; ++row) {
      const cell = cellsForColumn[row];
      if (cell !== undefined) {
        rowStrings[row] += cell.padStart(colWidth);
      }
    }

    totalTableWidth += colWidth;
    if (totalTableWidth >= fillToWidth) {
      for (let row = 0; row < rowStrings.length; ++row) {
        if (cellsForColumn[row] !== undefined) {
          rowStrings[row] += ' ...';
        }
      }
      break;
    }
  }
  return rowStrings.join('\n');
}