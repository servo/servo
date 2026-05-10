// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: ZonedDateTime constructor accepts link names as time zone ID input
features: [Temporal, canonical-tz]
---*/

const testCases = [
  "Africa/Asmera",  // Link    Africa/Nairobi          Africa/Asmera
  "Africa/Timbuktu",  // Link    Africa/Abidjan          Africa/Timbuktu
  "America/Argentina/ComodRivadavia",  // Link    America/Argentina/Catamarca     America/Argentina/ComodRivadavia
  "America/Atka",  // Link    America/Adak            America/Atka
  "America/Buenos_Aires",  // Link    America/Argentina/Buenos_Aires  America/Buenos_Aires
  "America/Catamarca",  // Link    America/Argentina/Catamarca     America/Catamarca
  "America/Coral_Harbour",  // Link    America/Panama          America/Coral_Harbour
  "America/Cordoba",  // Link    America/Argentina/Cordoba       America/Cordoba
  "America/Ensenada",  // Link    America/Tijuana         America/Ensenada
  "America/Fort_Wayne",  // Link    America/Indiana/Indianapolis    America/Fort_Wayne
  "America/Godthab",  // Link    America/Nuuk            America/Godthab
  "America/Indianapolis",  // Link    America/Indiana/Indianapolis    America/Indianapolis
  "America/Jujuy",  // Link    America/Argentina/Jujuy America/Jujuy
  "America/Knox_IN",  // Link    America/Indiana/Knox    America/Knox_IN
  "America/Louisville",  // Link    America/Kentucky/Louisville     America/Louisville
  "America/Mendoza",  // Link    America/Argentina/Mendoza       America/Mendoza
  "America/Montreal",  // Link    America/Toronto         America/Montreal
  "America/Porto_Acre",  // Link    America/Rio_Branco      America/Porto_Acre
  "America/Rosario",  // Link    America/Argentina/Cordoba       America/Rosario
  "America/Santa_Isabel",  // Link    America/Tijuana         America/Santa_Isabel
  "America/Shiprock",  // Link    America/Denver          America/Shiprock
  "America/Virgin",  // Link    America/Puerto_Rico     America/Virgin
  "Antarctica/South_Pole",  // Link    Pacific/Auckland        Antarctica/South_Pole
  "Asia/Ashkhabad",  // Link    Asia/Ashgabat           Asia/Ashkhabad
  "Asia/Calcutta",  // Link    Asia/Kolkata            Asia/Calcutta
  "Asia/Chongqing",  // Link    Asia/Shanghai           Asia/Chongqing
  "Asia/Chungking",  // Link    Asia/Shanghai           Asia/Chungking
  "Asia/Dacca",  // Link    Asia/Dhaka              Asia/Dacca
  "Asia/Harbin",  // Link    Asia/Shanghai           Asia/Harbin
  "Asia/Kashgar",  // Link    Asia/Urumqi             Asia/Kashgar
  "Asia/Katmandu",  // Link    Asia/Kathmandu          Asia/Katmandu
  "Asia/Macao",  // Link    Asia/Macau              Asia/Macao
  "Asia/Rangoon",  // Link    Asia/Yangon             Asia/Rangoon
  "Asia/Saigon",  // Link    Asia/Ho_Chi_Minh        Asia/Saigon
  "Asia/Tel_Aviv",  // Link    Asia/Jerusalem          Asia/Tel_Aviv
  "Asia/Thimbu",  // Link    Asia/Thimphu            Asia/Thimbu
  "Asia/Ujung_Pandang",  // Link    Asia/Makassar           Asia/Ujung_Pandang
  "Asia/Ulan_Bator",  // Link    Asia/Ulaanbaatar        Asia/Ulan_Bator
  "Atlantic/Faeroe",  // Link    Atlantic/Faroe          Atlantic/Faeroe
  "Atlantic/Jan_Mayen",  // Link    Europe/Oslo             Atlantic/Jan_Mayen
  "Australia/ACT",  // Link    Australia/Sydney        Australia/ACT
  "Australia/Canberra",  // Link    Australia/Sydney        Australia/Canberra
  "Australia/Currie",  // Link    Australia/Hobart        Australia/Currie
  "Australia/LHI",  // Link    Australia/Lord_Howe     Australia/LHI
  "Australia/NSW",  // Link    Australia/Sydney        Australia/NSW
  "Australia/North",  // Link    Australia/Darwin        Australia/North
  "Australia/Queensland",  // Link    Australia/Brisbane      Australia/Queensland
  "Australia/South",  // Link    Australia/Adelaide      Australia/South
  "Australia/Tasmania",  // Link    Australia/Hobart        Australia/Tasmania
  "Australia/Victoria",  // Link    Australia/Melbourne     Australia/Victoria
  "Australia/West",  // Link    Australia/Perth         Australia/West
  "Australia/Yancowinna",  // Link    Australia/Broken_Hill   Australia/Yancowinna
  "Brazil/Acre",  // Link    America/Rio_Branco      Brazil/Acre
  "Brazil/DeNoronha",  // Link    America/Noronha         Brazil/DeNoronha
  "Brazil/East",  // Link    America/Sao_Paulo       Brazil/East
  "Brazil/West",  // Link    America/Manaus          Brazil/West
  "Canada/Atlantic",  // Link    America/Halifax         Canada/Atlantic
  "Canada/Central",  // Link    America/Winnipeg        Canada/Central
  "Canada/Eastern",  // Link    America/Toronto         Canada/Eastern
  "Canada/Mountain",  // Link    America/Edmonton        Canada/Mountain
  "Canada/Newfoundland",  // Link    America/St_Johns        Canada/Newfoundland
  "Canada/Pacific",  // Link    America/Vancouver       Canada/Pacific
  "Canada/Saskatchewan",  // Link    America/Regina          Canada/Saskatchewan
  "Canada/Yukon",  // Link    America/Whitehorse      Canada/Yukon
  "Chile/Continental",  // Link    America/Santiago        Chile/Continental
  "Chile/EasterIsland",  // Link    Pacific/Easter          Chile/EasterIsland
  "Cuba",  // Link    America/Havana          Cuba
  "Egypt",  // Link    Africa/Cairo            Egypt
  "Eire",  // Link    Europe/Dublin           Eire
  "Etc/UCT",  // Link    Etc/UTC                 Etc/UCT
  "Europe/Belfast",  // Link    Europe/London           Europe/Belfast
  "Europe/Kiev",  // Link    Europe/Kyiv             Europe/Kiev
  "Europe/Tiraspol",  // Link    Europe/Chisinau         Europe/Tiraspol
  "GB",  // Link    Europe/London           GB
  "GB-Eire",  // Link    Europe/London           GB-Eire
  "GMT+0",  // Link    Etc/GMT                 GMT+0
  "GMT-0",  // Link    Etc/GMT                 GMT-0
  "GMT0",  // Link    Etc/GMT                 GMT0
  "Greenwich",  // Link    Etc/GMT                 Greenwich
  "Hongkong",  // Link    Asia/Hong_Kong          Hongkong
  "Iceland",  // Link    Atlantic/Reykjavik      Iceland
  "Iran",  // Link    Asia/Tehran             Iran
  "Israel",  // Link    Asia/Jerusalem          Israel
  "Jamaica",  // Link    America/Jamaica         Jamaica
  "Japan",  // Link    Asia/Tokyo              Japan
  "Kwajalein",  // Link    Pacific/Kwajalein       Kwajalein
  "Libya",  // Link    Africa/Tripoli          Libya
  "Mexico/BajaNorte",  // Link    America/Tijuana         Mexico/BajaNorte
  "Mexico/BajaSur",  // Link    America/Mazatlan        Mexico/BajaSur
  "Mexico/General",  // Link    America/Mexico_City     Mexico/General
  "NZ",  // Link    Pacific/Auckland        NZ
  "NZ-CHAT",  // Link    Pacific/Chatham         NZ-CHAT
  "Navajo",  // Link    America/Denver          Navajo
  "PRC",  // Link    Asia/Shanghai           PRC
  "Pacific/Enderbury",  // Link    Pacific/Kanton          Pacific/Enderbury
  "Pacific/Johnston",  // Link    Pacific/Honolulu        Pacific/Johnston
  "Pacific/Ponape",  // Link    Pacific/Pohnpei         Pacific/Ponape
  "Pacific/Samoa",  // Link    Pacific/Pago_Pago       Pacific/Samoa
  "Pacific/Truk",  // Link    Pacific/Chuuk           Pacific/Truk
  "Pacific/Yap",  // Link    Pacific/Chuuk           Pacific/Yap
  "Poland",  // Link    Europe/Warsaw           Poland
  "Portugal",  // Link    Europe/Lisbon           Portugal
  "ROC",  // Link    Asia/Taipei             ROC
  "ROK",  // Link    Asia/Seoul              ROK
  "Singapore",  // Link    Asia/Singapore          Singapore
  "Turkey",  // Link    Europe/Istanbul         Turkey
  "UCT",  // Link    Etc/UTC                 UCT
  "US/Alaska",  // Link    America/Anchorage       US/Alaska
  "US/Aleutian",  // Link    America/Adak            US/Aleutian
  "US/Arizona",  // Link    America/Phoenix         US/Arizona
  "US/Central",  // Link    America/Chicago         US/Central
  "US/East-Indiana",  // Link    America/Indiana/Indianapolis    US/East-Indiana
  "US/Eastern",  // Link    America/New_York        US/Eastern
  "US/Hawaii",  // Link    Pacific/Honolulu        US/Hawaii
  "US/Indiana-Starke",  // Link    America/Indiana/Knox    US/Indiana-Starke
  "US/Michigan",  // Link    America/Detroit         US/Michigan
  "US/Mountain",  // Link    America/Denver          US/Mountain
  "US/Pacific",  // Link    America/Los_Angeles     US/Pacific
  "US/Samoa",  // Link    Pacific/Pago_Pago       US/Samoa
  "UTC",  // Link    Etc/UTC                 UTC
  "Universal",  // Link    Etc/UTC                 Universal
  "W-SU",  // Link    Europe/Moscow           W-SU
  "Zulu",  // Link    Etc/UTC                 Zulu
];

for (let id of testCases) {
  const instance = new Temporal.ZonedDateTime(0n, id);
  assert.sameValue(instance.timeZoneId, id);
}
