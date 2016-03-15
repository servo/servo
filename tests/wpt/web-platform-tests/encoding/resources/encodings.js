// Based on https://dom.spec.whatwg.org/#dom-document-characterset

var compatibility_names = {
  "utf-8": "UTF-8",
  "ibm866": "IBM866",
  "iso-8859-2": "ISO-8859-2",
  "iso-8859-3": "ISO-8859-3",
  "iso-8859-4": "ISO-8859-4",
  "iso-8859-5": "ISO-8859-5",
  "iso-8859-6": "ISO-8859-6",
  "iso-8859-7": "ISO-8859-7",
  "iso-8859-8": "ISO-8859-8",
  "iso-8859-8-i": "ISO-8859-8-I",
  "iso-8859-10": "ISO-8859-10",
  "iso-8859-13": "ISO-8859-13",
  "iso-8859-14": "ISO-8859-14",
  "iso-8859-15": "ISO-8859-15",
  "iso-8859-16": "ISO-8859-16",
  "koi8-r": "KOI8-R",
  "koi8-u": "KOI8-U",
  "gbk": "GBK",
  "big5": "Big5",
  "euc-jp": "EUC-JP",
  "iso-2022-jp": "ISO-2022-JP",
  "shift_jis": "Shift_JIS",
  "euc-kr": "EUC-KR",
  "utf-16be": "UTF-16BE",
  "utf-16le": "UTF-16LE"
};

// Based on https://encoding.spec.whatwg.org/

var utf_encodings = ['utf-8', 'utf-16le', 'utf-16be'];

var encodings_table =
[
  {
    "encodings": [
      {
        "labels": [
          "unicode-1-1-utf-8",
          "utf-8",
          "utf8"
        ],
        "name": "utf-8"
      }
    ],
    "heading": "The Encoding"
  },
  {
    "encodings": [
      {
        "labels": [
          "866",
          "cp866",
          "csibm866",
          "ibm866"
        ],
        "name": "ibm866"
      },
      {
        "labels": [
          "csisolatin2",
          "iso-8859-2",
          "iso-ir-101",
          "iso8859-2",
          "iso88592",
          "iso_8859-2",
          "iso_8859-2:1987",
          "l2",
          "latin2"
        ],
        "name": "iso-8859-2"
      },
      {
        "labels": [
          "csisolatin3",
          "iso-8859-3",
          "iso-ir-109",
          "iso8859-3",
          "iso88593",
          "iso_8859-3",
          "iso_8859-3:1988",
          "l3",
          "latin3"
        ],
        "name": "iso-8859-3"
      },
      {
        "labels": [
          "csisolatin4",
          "iso-8859-4",
          "iso-ir-110",
          "iso8859-4",
          "iso88594",
          "iso_8859-4",
          "iso_8859-4:1988",
          "l4",
          "latin4"
        ],
        "name": "iso-8859-4"
      },
      {
        "labels": [
          "csisolatincyrillic",
          "cyrillic",
          "iso-8859-5",
          "iso-ir-144",
          "iso8859-5",
          "iso88595",
          "iso_8859-5",
          "iso_8859-5:1988"
        ],
        "name": "iso-8859-5"
      },
      {
        "labels": [
          "arabic",
          "asmo-708",
          "csiso88596e",
          "csiso88596i",
          "csisolatinarabic",
          "ecma-114",
          "iso-8859-6",
          "iso-8859-6-e",
          "iso-8859-6-i",
          "iso-ir-127",
          "iso8859-6",
          "iso88596",
          "iso_8859-6",
          "iso_8859-6:1987"
        ],
        "name": "iso-8859-6"
      },
      {
        "labels": [
          "csisolatingreek",
          "ecma-118",
          "elot_928",
          "greek",
          "greek8",
          "iso-8859-7",
          "iso-ir-126",
          "iso8859-7",
          "iso88597",
          "iso_8859-7",
          "iso_8859-7:1987",
          "sun_eu_greek"
        ],
        "name": "iso-8859-7"
      },
      {
        "labels": [
          "csiso88598e",
          "csisolatinhebrew",
          "hebrew",
          "iso-8859-8",
          "iso-8859-8-e",
          "iso-ir-138",
          "iso8859-8",
          "iso88598",
          "iso_8859-8",
          "iso_8859-8:1988",
          "visual"
        ],
        "name": "iso-8859-8"
      },
      {
        "labels": [
          "csiso88598i",
          "iso-8859-8-i",
          "logical"
        ],
        "name": "iso-8859-8-i"
      },
      {
        "labels": [
          "csisolatin6",
          "iso-8859-10",
          "iso-ir-157",
          "iso8859-10",
          "iso885910",
          "l6",
          "latin6"
        ],
        "name": "iso-8859-10"
      },
      {
        "labels": [
          "iso-8859-13",
          "iso8859-13",
          "iso885913"
        ],
        "name": "iso-8859-13"
      },
      {
        "labels": [
          "iso-8859-14",
          "iso8859-14",
          "iso885914"
        ],
        "name": "iso-8859-14"
      },
      {
        "labels": [
          "csisolatin9",
          "iso-8859-15",
          "iso8859-15",
          "iso885915",
          "iso_8859-15",
          "l9"
        ],
        "name": "iso-8859-15"
      },
      {
        "labels": [
          "iso-8859-16"
        ],
        "name": "iso-8859-16"
      },
      {
        "labels": [
          "cskoi8r",
          "koi",
          "koi8",
          "koi8-r",
          "koi8_r"
        ],
        "name": "koi8-r"
      },
      {
        "labels": [
          "koi8-u"
        ],
        "name": "koi8-u"
      },
      {
        "labels": [
          "csmacintosh",
          "mac",
          "macintosh",
          "x-mac-roman"
        ],
        "name": "macintosh"
      },
      {
        "labels": [
          "dos-874",
          "iso-8859-11",
          "iso8859-11",
          "iso885911",
          "tis-620",
          "windows-874"
        ],
        "name": "windows-874"
      },
      {
        "labels": [
          "cp1250",
          "windows-1250",
          "x-cp1250"
        ],
        "name": "windows-1250"
      },
      {
        "labels": [
          "cp1251",
          "windows-1251",
          "x-cp1251"
        ],
        "name": "windows-1251"
      },
      {
        "labels": [
          "ansi_x3.4-1968",
          "ascii",
          "cp1252",
          "cp819",
          "csisolatin1",
          "ibm819",
          "iso-8859-1",
          "iso-ir-100",
          "iso8859-1",
          "iso88591",
          "iso_8859-1",
          "iso_8859-1:1987",
          "l1",
          "latin1",
          "us-ascii",
          "windows-1252",
          "x-cp1252"
        ],
        "name": "windows-1252"
      },
      {
        "labels": [
          "cp1253",
          "windows-1253",
          "x-cp1253"
        ],
        "name": "windows-1253"
      },
      {
        "labels": [
          "cp1254",
          "csisolatin5",
          "iso-8859-9",
          "iso-ir-148",
          "iso8859-9",
          "iso88599",
          "iso_8859-9",
          "iso_8859-9:1989",
          "l5",
          "latin5",
          "windows-1254",
          "x-cp1254"
        ],
        "name": "windows-1254"
      },
      {
        "labels": [
          "cp1255",
          "windows-1255",
          "x-cp1255"
        ],
        "name": "windows-1255"
      },
      {
        "labels": [
          "cp1256",
          "windows-1256",
          "x-cp1256"
        ],
        "name": "windows-1256"
      },
      {
        "labels": [
          "cp1257",
          "windows-1257",
          "x-cp1257"
        ],
        "name": "windows-1257"
      },
      {
        "labels": [
          "cp1258",
          "windows-1258",
          "x-cp1258"
        ],
        "name": "windows-1258"
      },
      {
        "labels": [
          "x-mac-cyrillic",
          "x-mac-ukrainian"
        ],
        "name": "x-mac-cyrillic"
      }
    ],
    "heading": "Legacy single-byte encodings"
  },
  {
    "encodings": [
      {
        "labels": [
          "chinese",
          "csgb2312",
          "csiso58gb231280",
          "gb2312",
          "gb_2312",
          "gb_2312-80",
          "gbk",
          "iso-ir-58",
          "x-gbk"
        ],
        "name": "gbk"
      },
      {
        "labels": [
          "gb18030"
        ],
        "name": "gb18030"
      }
    ],
    "heading": "Legacy multi-byte Chinese (simplified) encodings"
  },
  {
    "encodings": [
      {
        "labels": [
          "big5",
          "big5-hkscs",
          "cn-big5",
          "csbig5",
          "x-x-big5"
        ],
        "name": "big5"
      }
    ],
    "heading": "Legacy multi-byte Chinese (traditional) encodings"
  },
  {
    "encodings": [
      {
        "labels": [
          "cseucpkdfmtjapanese",
          "euc-jp",
          "x-euc-jp"
        ],
        "name": "euc-jp"
      },
      {
        "labels": [
          "csiso2022jp",
          "iso-2022-jp"
        ],
        "name": "iso-2022-jp"
      },
      {
        "labels": [
          "csshiftjis",
          "ms932",
          "ms_kanji",
          "shift-jis",
          "shift_jis",
          "sjis",
          "windows-31j",
          "x-sjis"
        ],
        "name": "shift_jis"
      }
    ],
    "heading": "Legacy multi-byte Japanese encodings"
  },
  {
    "encodings": [
      {
        "labels": [
          "cseuckr",
          "csksc56011987",
          "euc-kr",
          "iso-ir-149",
          "korean",
          "ks_c_5601-1987",
          "ks_c_5601-1989",
          "ksc5601",
          "ksc_5601",
          "windows-949"
        ],
        "name": "euc-kr"
      }
    ],
    "heading": "Legacy multi-byte Korean encodings"
  },
  {
    "encodings": [
      {
        "labels": [
          "csiso2022kr",
          "hz-gb-2312",
          "iso-2022-cn",
          "iso-2022-cn-ext",
          "iso-2022-kr"
        ],
        "name": "replacement"
      },
      {
        "labels": [
          "utf-16be"
        ],
        "name": "utf-16be"
      },
      {
        "labels": [
          "utf-16",
          "utf-16le"
        ],
        "name": "utf-16le"
      },
      {
        "labels": [
          "x-user-defined"
        ],
        "name": "x-user-defined"
      }
    ],
    "heading": "Legacy miscellaneous encodings"
  }
]
;
