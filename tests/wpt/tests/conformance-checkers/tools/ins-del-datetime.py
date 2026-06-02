# -*- coding: utf-8 -*-
import os
ccdir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
template = """<!DOCTYPE html>
<meta charset=utf-8>
"""
errors = {
    "date-year-0000": "0000-12-09",
    "date-month-00": "2002-00-15",
    "date-month-13": "2002-13-15",
    "date-0005-02-29": "0005-02-29",
    "date-1969-02-29": "1969-02-29",
    "date-1900-02-29": "1900-02-29",
    "date-2100-02-29": "2100-02-29",
    "date-2200-02-29": "2200-02-29",
    "date-2014-02-29": "2014-02-29",
    "date-day-04-31": "2002-04-31",
    "date-day-06-31": "2002-06-31",
    "date-day-09-31": "2002-09-31",
    "date-day-11-31": "2002-11-31",
    "date-day-01-32": "2002-01-32",
    "date-day-03-32": "2002-03-32",
    "date-day-05-32": "2002-05-32",
    "date-day-07-32": "2002-07-32",
    "date-day-08-32": "2002-08-32",
    "date-day-10-32": "2002-10-32",
    "date-day-12-32": "2002-12-32",
    "date-iso8601-YYYYMMDD-no-hyphen": "20020929",
    "date-leading-whitespace": " 2002-09-29",
    "date-trailing-whitespace": "2002-09-29 ",
    "date-month-one-digit": "2002-9-29",
    "date-month-three-digits": "2002-011-29",
    "date-year-three-digits": "782-09-29",
    "date-day-one-digit": "2002-09-9",
    "date-day-three-digits": "2002-11-009",
    "date-day-missing-separator": "2014-0220",
    "date-month-missing-separator": "201402-20",
    "date-non-ascii-digit": "2002-09-2９",
    "date-trailing-U+0000": "2002-09-29&#x0000;",
    "date-trailing-pile-of-poo": "2002-09-29💩",
    "date-wrong-day-separator": "2014-02:20",
    "date-wrong-month-separator": "2014:02-20",
    "date-year-negative": "-2002-09-29",
    "date-leading-bom": "﻿2002-09-29",
    "global-date-and-time-60-minutes": "2011-11-12T00:60:00+08:00",
    "global-date-and-time-60-seconds": "2011-11-12T00:00:60+08:00",
    "global-date-and-time-2400": "2011-11-12T24:00:00+08:00",
    "global-date-and-time-space-before-timezone": "2011-11-12T06:54:39 08:00",
    "global-date-and-time-hour-one-digit": "2011-11-12T6:54:39-08:00",
    "global-date-and-time-hour-three-digits": "2011-11-12T016:54:39-08:00",
    "global-date-and-time-minutes-one-digit": "2011-11-12T16:4:39-08:00",
    "global-date-and-time-minutes-three-digits": "2011-11-12T16:354:39-08:00",
    "global-date-and-time-seconds-one-digit": "2011-11-12T16:54:9-08:00",
    "global-date-and-time-seconds-three-digits": "2011-11-12T16:54:039-08:00",
    "global-date-and-time-timezone-with-seconds": "2011-11-12T06:54:39-08:00:00",
    "global-date-and-time-timezone-60-minutes": "2011-11-12T06:54:39-08:60",
    "global-date-and-time-timezone-one-digit-hour": "2011-11-12T06:54:39-5:00",
    "global-date-and-time-timezone-one-digit-minute": "2011-11-12T06:54:39-05:0",
    "global-date-and-time-timezone-three-digit-hour": "2011-11-12T06:54:39-005:00",
    "global-date-and-time-timezone-three-digit-minute": "2011-11-12T06:54:39-05:000",
    "global-date-and-time-nbsp": "2011-11-12 14:54Z",
    "global-date-and-time-missing-minutes-separator": "2011-11-12T1454Z",
    "global-date-and-time-missing-seconds-separator": "2011-11-12T14:5439Z",
    "global-date-and-time-wrong-minutes-separator": "2011-11-12T14-54Z",
    "global-date-and-time-wrong-seconds-separator": "2011-11-12T14:54-39Z",
    "global-date-and-time-lowercase-z": "2011-11-12T14:54z",
    "global-date-and-time-with-both-T-and-space": "2011-11-12T 14:54Z",
    "global-date-and-time-zero-digit-fraction": "2011-11-12T06:54:39.-08:00",
    "global-date-and-time-four-digit-fraction": "2011-11-12T06:54:39.9291-08:00",
    "global-date-and-time-bad-fraction-separator": "2011-11-12T14:54:39,929+0000",
    "global-date-and-time-timezone-non-T-character": "2011-11-12+14:54Z",
    "global-date-and-time-timezone-lowercase-t": "2011-11-12t14:54Z",
    "global-date-and-time-timezone-multiple-spaces": "2011-11-12  14:54Z",
    "global-date-and-time-timezone-offset-space-start": "2011-11-12T06:54:39.929 08:00",
    "global-date-and-time-timezone-offset-colon-start": "2011-11-12T06:54:39.929:08:00",
    "global-date-and-time-timezone-plus-2400": "2011-11-12T06:54:39-24:00",
    "global-date-and-time-timezone-minus-2400": "2011-11-12T06:54:39-24:00",
    "global-date-and-time-timezone-iso8601-two-digit": "2011-11-12T06:54:39-08",
    "global-date-and-time-iso8601-hhmmss-no-colon": "2011-11-12T145439Z",
    "global-date-and-time-iso8601-hhmm-no-colon": "2011-11-12T1454Z",
    "global-date-and-time-iso8601-hh": "2011-11-12T14Z",
    "year": "2006",
    "yearless-date": "07-15",
    "month": "2011-11",
    "week": "2011-W46",
    "time": "14:54:39",
    "local-date-and-time": "2011-11-12T14:54",
    "duration-P-form": "PT4H18M3S",
    "duration-time-component": "4h 18m 3s",
}

warnings = {
    "global-date-and-time-timezone-plus-1500": "2011-11-12T00:00:00+1500",
    "global-date-and-time-timezone-minus-1300": "2011-11-12T00:00:00-1300",
    "global-date-and-time-timezone-minutes-15": "2011-11-12T00:00:00+08:15",
    "date-0214-09-29": "0214-09-29",
    "date-20014-09-29": "20014-09-29",
    "date-0004-02-29": "0004-02-29",
    "date-year-five-digits": "12014-09-29",
}

non_errors = {
    "date": "2002-09-29",
    "date-2000-02-29": "2000-02-29",
    "date-2400-02-29": "2400-02-29",
    "date-1968-02-29": "1968-02-29",
    "date-1900-02-28": "1900-02-28",
    "date-2100-02-28": "2100-02-28",
    "date-2200-02-28": "2200-02-28",
    "date-2014-02-28": "2014-02-28",
    "date-day-01-31": "2002-01-31",
    "date-day-03-31": "2002-03-31",
    "date-day-05-31": "2002-05-31",
    "date-day-07-31": "2002-07-31",
    "date-day-08-31": "2002-08-31",
    "date-day-10-31": "2002-10-31",
    "date-day-12-31": "2002-12-31",
    "date-day-04-30": "2002-04-30",
    "date-day-06-30": "2002-06-30",
    "date-day-09-30": "2002-09-30",
    "date-day-11-30": "2002-11-30",
    "global-date-and-time-no-seconds": "2011-11-12T14:54Z",
    "global-date-and-time-with-seconds": "2011-11-12T14:54:39+0000",
    "global-date-and-time-with-one-digit-fraction": "2011-11-12T06:54:39.9-08:00",
    "global-date-and-time-with-two-digit-fraction": "2011-11-12T06:54:39.92+07:00",
    "global-date-and-time-with-three-digit-fraction": "2011-11-12T06:54:39.929-06:00",
    "global-date-and-time-space": "2011-11-12 14:54Z",
    "global-date-and-time-timezone": "2011-11-12T06:54:39+0900",
    "global-date-and-time-timezone-30": "2011-11-12T06:54:39-0830",
    "global-date-and-time-timezone-45": "2011-11-12T06:54:39-0845",
    "global-date-and-time-timezone-with-colon": "2011-11-12T06:54:39-08:00",
    "global-date-and-time-timezone-without-colon": "2011-11-12T06:54:39-0800",
}

for key in errors.keys():
    error = errors[key]
    template_ins = template
    template_del = template
    template_ins += '<title>%s</title>\n' % key
    template_del += '<title>%s</title>\n' % key
    template_ins += '<ins datetime="%s"></ins>' % errors[key]
    template_del += '<del datetime="%s"></del>' % errors[key]
    ins_file = open(os.path.join(ccdir, "html/elements/ins/%s-novalid.html" % key), 'w')
    ins_file.write(template_ins)
    ins_file.close()
    del_file = open(os.path.join(ccdir, "html/elements/del/%s-novalid.html" % key), 'w')
    del_file.write(template_del)
    del_file.close()

for key in warnings.keys():
    non_error = warnings[key]
    template_ins = template
    template_del = template
    template_ins += '<title>%s</title>\n' % key
    template_del += '<title>%s</title>\n' % key
    template_ins += '<ins datetime="%s"></ins>' % warnings[key]
    template_del += '<del datetime="%s"></del>' % warnings[key]
    ins_file = open(os.path.join(ccdir, "html/elements/ins/%s-haswarn.html" % key), 'w')
    ins_file.write(template_ins)
    ins_file.close()
    del_file = open(os.path.join(ccdir, "html/elements/del/%s-haswarn.html" % key), 'w')
    del_file.write(template_del)
    del_file.close()

ins_file = open(os.path.join(ccdir, "html/elements/ins/datetime-isvalid.html"), 'w')
del_file = open(os.path.join(ccdir, "html/elements/del/datetime-isvalid.html"), 'w')
ins_file.write(template + '<title>valid datetime</title>\n')
del_file.write(template + '<title>valid datetime</title>\n')
for key in non_errors.keys():
    non_error = non_errors[key]
    ins_file.write('<ins datetime="%s"></ins> <!-- %s -->\n' % (non_errors[key], key))
    del_file.write('<del datetime="%s"></del> <!-- %s -->\n' % (non_errors[key], key))
ins_file.close()
del_file.close()
# vim: ts=4:sw=4
