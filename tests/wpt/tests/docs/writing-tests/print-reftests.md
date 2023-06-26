# Print Reftests

Print reftests are like ordinary [reftests](reftests), except that the
output is rendered to pagninated form and then compared page-by-page
with the reference.

Print reftests are distinguished by the string `-print` in the
filename immediately before the extension, or by being under a
directory named `print`. Examples:

- `css/css-foo/bar-print.html` is a print reftest
- `css/css-foo/print/bar.html` is a print reftest
- `css/css-foo/bar-print-001.html` is **not** a print reftest


Like ordinary reftests, the reference is specified using a `<link
rel=match>` element.

The default page size for print reftests is 12.7 cm by 7.62 cm (5
inches by 3 inches).

All the features of ordinary reftests also work with print reftests
including [fuzzy matching](reftests.html#fuzzy-matching). Any fuzzy
specifier applies to each image comparison performed i.e. separately
for each page.

## Page Ranges

In some cases it may be desirable to only compare a subset of the
output pages in the reftest. This is possible using
```
<meta name=reftest-pages content=[range-specifier]>
```
Where a range specifier has the form
```
range-specifier = <specifier-item> ["," <specifier-item>]*
specifier-item = <int> | <int>? "-" <int>?
```

For example to specify rendering pages 1 and 2, 4, 6 and 7, and 9 and
10 of a 10 page page document one could write:

```
<meta name=reftest-pages content="-2,4,6,7,9-">
```
