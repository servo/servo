// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: ZonedDateTime constructor accepts link names as time zone ID input
features: [Temporal, canonical-tz]
---*/

const asiaTestCases = {
  "Europe/Nicosia": "Asia/Nicosia",
  "Asia/Ashkhabad": "Asia/Ashgabat",
  "Asia/Calcutta": "Asia/Kolkata",
  "Asia/Choibalsan": "Asia/Ulaanbaatar",
  "Asia/Chongqing": "Asia/Shanghai",
  "Asia/Chungking": "Asia/Shanghai",
  "Asia/Dacca": "Asia/Dhaka",
  "Asia/Harbin": "Asia/Shanghai",
  "Asia/Istanbul": "Europe/Istanbul",
  "Asia/Kashgar": "Asia/Urumqi",
  "Asia/Katmandu": "Asia/Kathmandu",
  "Asia/Macao": "Asia/Macau",
  "Asia/Rangoon": "Asia/Yangon",
  "Asia/Saigon": "Asia/Ho_Chi_Minh",
  "Asia/Tel_Aviv": "Asia/Jerusalem",
  "Asia/Thimbu": "Asia/Thimphu",
  "Asia/Ujung_Pandang": "Asia/Makassar",
  "Asia/Ulan_Bator": "Asia/Ulaanbaatar",
};

const asiaHistoricalTestCases = [
  "Antarctica/Syowa", // Link Asia/Riyadh Antarctica/Syowa
  "Asia/Aden", // Link Asia/Riyadh Asia/Aden      # Yemen
  "Asia/Bahrain", // Link Asia/Qatar Asia/Bahrain
  "Asia/Kuwait", // Link Asia/Riyadh Asia/Kuwait
  "Asia/Phnom_Penh", // Link Asia/Bangkok Asia/Phnom_Penh       # Cambodia
  "Asia/Vientiane", // Link Asia/Bangkok Asia/Vientiane        # Laos
  "Asia/Muscat", // Link Asia/Dubai Asia/Muscat     # Oman
];

const africaTestCases = {
  "Africa/Asmera": "Africa/Asmara",
  "Africa/Timbuktu": "Africa/Bamako",
};

const africaHistoricalTestCases = [
  "Africa/Accra",  // Link Africa/Abidjan Africa/Accra        # Ghana
  "Africa/Bamako",  // Link Africa/Abidjan Africa/Bamako       # Mali
  "Africa/Banjul",  // Link Africa/Abidjan Africa/Banjul       # The Gambia
  "Africa/Conakry",  // Link Africa/Abidjan Africa/Conakry      # Guinea
  "Africa/Dakar",  // Link Africa/Abidjan Africa/Dakar        # Senegal
  "Africa/Freetown",  // Link Africa/Abidjan Africa/Freetown     # Sierra Leone
  "Africa/Lome",  // Link Africa/Abidjan Africa/Lome         # Togo
  "Africa/Nouakchott",  // Link Africa/Abidjan Africa/Nouakchott   # Mauritania
  "Africa/Ouagadougou",  // Link Africa/Abidjan Africa/Ouagadougou  # Burkina Faso
  "Atlantic/St_Helena",  // Link Africa/Abidjan Atlantic/St_Helena  # St Helena
  "Africa/Addis_Ababa",  // Link Africa/Nairobi Africa/Addis_Ababa   # Ethiopia
  "Africa/Asmara",  // Link Africa/Nairobi Africa/Asmara        # Eritrea
  "Africa/Dar_es_Salaam",  // Link Africa/Nairobi Africa/Dar_es_Salaam # Tanzania
  "Africa/Djibouti",  // Link Africa/Nairobi Africa/Djibouti
  "Africa/Kampala",  // Link Africa/Nairobi Africa/Kampala       # Uganda
  "Africa/Mogadishu",  // Link Africa/Nairobi Africa/Mogadishu     # Somalia
  "Indian/Antananarivo",  // Link Africa/Nairobi Indian/Antananarivo  # Madagascar
  "Indian/Comoro",  // Link Africa/Nairobi Indian/Comoro
  "Indian/Mayotte",  // Link Africa/Nairobi Indian/Mayotte
  "Africa/Blantyre",  // Link Africa/Maputo Africa/Blantyre      # Malawi
  "Africa/Bujumbura",  // Link Africa/Maputo Africa/Bujumbura     # Burundi
  "Africa/Gaborone",  // Link Africa/Maputo Africa/Gaborone      # Botswana
  "Africa/Harare",  // Link Africa/Maputo Africa/Harare        # Zimbabwe
  "Africa/Kigali",  // Link Africa/Maputo Africa/Kigali        # Rwanda
  "Africa/Lubumbashi",  // Link Africa/Maputo Africa/Lubumbashi    # E Dem. Rep. of Congo
  "Africa/Lusaka",  // Link Africa/Maputo Africa/Lusaka        # Zambia
  "Africa/Bangui",  // Link Africa/Lagos Africa/Bangui         # Central African Republic
  "Africa/Brazzaville",  // Link Africa/Lagos Africa/Brazzaville    # Rep. of the Congo
  "Africa/Douala",  // Link Africa/Lagos Africa/Douala         # Cameroon
  "Africa/Kinshasa",  // Link Africa/Lagos Africa/Kinshasa       # Dem. Rep. of the Congo (west)
  "Africa/Libreville",  // Link Africa/Lagos Africa/Libreville     # Gabon
  "Africa/Luanda",  // Link Africa/Lagos Africa/Luanda         # Angola
  "Africa/Malabo",  // Link Africa/Lagos Africa/Malabo         # Equatorial Guinea
  "Africa/Niamey",  // Link Africa/Lagos Africa/Niamey         # Niger
  "Africa/Porto-Novo",  // Link Africa/Lagos Africa/Porto-Novo     # Benin
  "Africa/Maseru",  // Link Africa/Johannesburg Africa/Maseru  # Lesotho
  "Africa/Mbabane",  // Link Africa/Johannesburg Africa/Mbabane # Eswatini
];

const australasiaTestCases = {
  "Antarctica/South_Pole": "Antarctica/McMurdo",
  "Australia/ACT": "Australia/Sydney",
  "Australia/Canberra": "Australia/Sydney",
  "Australia/Currie": "Australia/Hobart",
  "Australia/LHI": "Australia/Lord_Howe",
  "Australia/NSW": "Australia/Sydney",
  "Australia/North": "Australia/Darwin",
  "Australia/Queensland": "Australia/Brisbane",
  "Australia/South": "Australia/Adelaide",
  "Australia/Tasmania": "Australia/Hobart",
  "Australia/Victoria": "Australia/Melbourne",
  "Australia/West": "Australia/Perth",
  "Australia/Yancowinna": "Australia/Broken_Hill",
  "Pacific/Enderbury": "Pacific/Kanton",
  "Pacific/Johnston": "Pacific/Honolulu",
  "Pacific/Ponape": "Pacific/Pohnpei",
  "Pacific/Samoa": "Pacific/Pago_Pago",
  "Pacific/Truk": "Pacific/Chuuk",
  "Pacific/Yap": "Pacific/Chuuk",
};

const australasiaHistoricalTestCases = [
  "Pacific/Saipan",  // Link Pacific/Guam Pacific/Saipan # N Mariana Is
  "Antarctica/McMurdo",  // Link Pacific/Auckland Antarctica/McMurdo
  "Antarctica/DumontDUrville",  // Link Pacific/Port_Moresby Antarctica/DumontDUrville
  "Pacific/Midway",  // Link Pacific/Pago_Pago Pacific/Midway # in US minor outlying islands
];

const europeTestCases = {
  "Europe/Belfast": "Europe/London",
  "Europe/Kiev": "Europe/Kyiv",
  "Europe/Nicosia": "Asia/Nicosia",
  "Europe/Tiraspol": "Europe/Chisinau",
  "Europe/Uzhgorod": "Europe/Kyiv",
  "Europe/Zaporozhye": "Europe/Kyiv",
};

const europeHistoricalTestCases = [
  "Europe/Jersey",  // Link    Europe/London   Europe/Jersey
  "Europe/Guernsey",  // Link    Europe/London   Europe/Guernsey
  "Europe/Isle_of_Man",  // Link    Europe/London   Europe/Isle_of_Man
  "Europe/Mariehamn",  // Link    Europe/Helsinki Europe/Mariehamn
  "Europe/Busingen",  // Link    Europe/Zurich   Europe/Busingen
  "Europe/Vatican",  // Link    Europe/Rome     Europe/Vatican
  "Europe/San_Marino",  // Link    Europe/Rome     Europe/San_Marino
  "Europe/Vaduz",  // Link Europe/Zurich Europe/Vaduz
  "Arctic/Longyearbyen",  // Link    Europe/Oslo     Arctic/Longyearbyen
  "Europe/Ljubljana",  // Link Europe/Belgrade Europe/Ljubljana   # Slovenia
  "Europe/Podgorica",  // Link Europe/Belgrade Europe/Podgorica   # Montenegro
  "Europe/Sarajevo",  // Link Europe/Belgrade Europe/Sarajevo    # Bosnia and Herzegovina
  "Europe/Skopje",  // Link Europe/Belgrade Europe/Skopje      # North Macedonia
  "Europe/Zagreb",  // Link Europe/Belgrade Europe/Zagreb      # Croatia
  "Europe/Bratislava",  // Link Europe/Prague Europe/Bratislava
  "Asia/Istanbul",  // Link    Europe/Istanbul Asia/Istanbul   # Istanbul is in both continents.
];

const northAmericaTestCases = {
  "America/Argentina/ComodRivadavia": "America/Argentina/Catamarca",
  "America/Atka": "America/Adak",
  "America/Buenos_Aires": "America/Argentina/Buenos_Aires",
  "America/Catamarca": "America/Argentina/Catamarca",
  "America/Coral_Harbour": "America/Atikokan",
  "America/Cordoba": "America/Argentina/Cordoba",
  "America/Ensenada": "America/Tijuana",
  "America/Fort_Wayne": "America/Indiana/Indianapolis",
  "America/Godthab": "America/Nuuk",
  "America/Indianapolis": "America/Indiana/Indianapolis",
  "America/Jujuy": "America/Argentina/Jujuy",
  "America/Knox_IN": "America/Indiana/Knox",
  "America/Louisville": "America/Kentucky/Louisville",
  "America/Mendoza": "America/Argentina/Mendoza",
  "America/Montreal": "America/Toronto",
  "America/Nipigon": "America/Toronto",
  "America/Pangnirtung": "America/Iqaluit",
  "America/Porto_Acre": "America/Rio_Branco",
  "America/Rainy_River": "America/Winnipeg",
  "America/Rosario": "America/Argentina/Cordoba",
  "America/Santa_Isabel": "America/Tijuana",
  "America/Shiprock": "America/Denver",
  "America/Thunder_Bay": "America/Toronto",
  "America/Virgin": "America/St_Thomas",
  "America/Yellowknife": "America/Edmonton",
  "US/Alaska": "America/Anchorage",
  "US/Aleutian": "America/Adak",
  "US/Arizona": "America/Phoenix",
  "US/Central": "America/Chicago",
  "US/East-Indiana": "America/Indiana/Indianapolis",
  "US/Eastern": "America/New_York",
  "US/Hawaii": "Pacific/Honolulu",
  "US/Indiana-Starke": "America/Indiana/Knox",
  "US/Michigan": "America/Detroit",
  "US/Mountain": "America/Denver",
  "US/Pacific": "America/Los_Angeles",
  "US/Samoa": "Pacific/Pago_Pago",
};

const northAmericaHistoricalTestCases = [
  "America/Creston",  // Link America/Phoenix America/Creston
  "America/Nassau",  // Link America/Toronto America/Nassau
  "America/Atikokan",  // Link America/Panama America/Atikokan
  "America/Cayman",  // Link America/Panama America/Cayman
  "America/Anguilla",  // Link America/Puerto_Rico America/Anguilla
  "America/Antigua",  // Link America/Puerto_Rico America/Antigua
  "America/Aruba",  // Link America/Puerto_Rico America/Aruba
  "America/Curacao",  // Link America/Puerto_Rico America/Curacao
  "America/Blanc-Sablon",  // Link America/Puerto_Rico America/Blanc-Sablon   # Quebec (Lower North Shore)
  "America/Dominica",  // Link America/Puerto_Rico America/Dominica
  "America/Grenada",  // Link America/Puerto_Rico America/Grenada
  "America/Guadeloupe",  // Link America/Puerto_Rico America/Guadeloupe
  "America/Kralendijk",  // Link America/Puerto_Rico America/Kralendijk     # Caribbean Netherlands
  "America/Lower_Princes",  // Link America/Puerto_Rico America/Lower_Princes  # Sint Maarten
  "America/Marigot",  // Link America/Puerto_Rico America/Marigot        # St Martin (French part)
  "America/Montserrat",  // Link America/Puerto_Rico America/Montserrat
  "America/Port_of_Spain",  // Link America/Puerto_Rico America/Port_of_Spain  # Trinidad & Tobago
  "America/St_Barthelemy",  // Link America/Puerto_Rico America/St_Barthelemy  # St Barthélemy
  "America/St_Kitts",  // Link America/Puerto_Rico America/St_Kitts       # St Kitts & Nevis
  "America/St_Lucia",  // Link America/Puerto_Rico America/St_Lucia
  "America/St_Thomas",  // Link America/Puerto_Rico America/St_Thomas      # Virgin Islands (US)
  "America/St_Vincent",  // Link America/Puerto_Rico America/St_Vincent
  "America/Tortola",  // Link America/Puerto_Rico America/Tortola        # Virgin Islands (UK)
];

const otherTestCases = {
  "Atlantic/Faeroe": "Atlantic/Faroe",
  "Atlantic/Jan_Mayen": "Arctic/Longyearbyen",
  "Brazil/Acre": "America/Rio_Branco",
  "Brazil/DeNoronha": "America/Noronha",
  "Brazil/East": "America/Sao_Paulo",
  "Brazil/West": "America/Manaus",
  "CET": "Europe/Brussels",
  "CST6CDT": "America/Chicago",
  "Canada/Atlantic": "America/Halifax",
  "Canada/Central": "America/Winnipeg",
  "Canada/Eastern": "America/Toronto",
  "Canada/Mountain": "America/Edmonton",
  "Canada/Newfoundland": "America/St_Johns",
  "Canada/Pacific": "America/Vancouver",
  "Canada/Saskatchewan": "America/Regina",
  "Canada/Yukon": "America/Whitehorse",
  "Chile/Continental": "America/Santiago",
  "Chile/EasterIsland": "Pacific/Easter",
  "Cuba": "America/Havana",
  "EET": "Europe/Athens",
  "EST": "America/Panama",
  "EST5EDT": "America/New_York",
  "Egypt": "Africa/Cairo",
  "Eire": "Europe/Dublin",
  "Etc/GMT": "UTC",
  "Etc/GMT+0": "UTC",
  "Etc/GMT-0": "UTC",
  "Etc/GMT0": "UTC",
  "Etc/Greenwich": "UTC",
  "Etc/UCT": "UTC",
  "Etc/UTC": "UTC",
  "Etc/Universal": "UTC",
  "Etc/Zulu": "UTC",
  "GB": "Europe/London",
  "GB-Eire": "Europe/London",
  "GMT": "UTC",
  "GMT+0": "UTC",
  "GMT-0": "UTC",
  "GMT0": "UTC",
  "Greenwich": "UTC",
  "HST": "Pacific/Honolulu",
  "Hongkong": "Asia/Hong_Kong",
  "Iceland": "Atlantic/Reykjavik",
  "Iran": "Asia/Tehran",
  "Israel": "Asia/Jerusalem",
  "Jamaica": "America/Jamaica",
  "Japan": "Asia/Tokyo",
  "Kwajalein": "Pacific/Kwajalein",
  "Libya": "Africa/Tripoli",
  "MET": "Europe/Brussels",
  "MST": "America/Phoenix",
  "MST7MDT": "America/Denver",
  "Mexico/BajaNorte": "America/Tijuana",
  "Mexico/BajaSur": "America/Mazatlan",
  "Mexico/General": "America/Mexico_City",
  "NZ": "Pacific/Auckland",
  "NZ-CHAT": "Pacific/Chatham",
  "Navajo": "America/Denver",
  "PRC": "Asia/Shanghai",
  "PST8PDT": "America/Los_Angeles",
  "Poland": "Europe/Warsaw",
  "Portugal": "Europe/Lisbon",
  "ROC": "Asia/Taipei",
  "ROK": "Asia/Seoul",
  "Singapore": "Asia/Singapore",
  "Turkey": "Europe/Istanbul",
  "UCT": "UTC",
  "Universal": "UTC",
  "W-SU": "Europe/Moscow",
  "WET": "Europe/Lisbon",
  "Zulu": "UTC",
};


let epochNanoseconds = [
  new Temporal.PlainDate(1900, 1, 1).toZonedDateTime("UTC").epochNanoseconds,
  new Temporal.PlainDate(1950, 1, 1).toZonedDateTime("UTC").epochNanoseconds,
  new Temporal.PlainDate(1960, 1, 1).toZonedDateTime("UTC").epochNanoseconds,
  new Temporal.PlainDate(1970, 1, 1).toZonedDateTime("UTC").epochNanoseconds,
  new Temporal.PlainDate(1980, 1, 1).toZonedDateTime("UTC").epochNanoseconds,
  new Temporal.PlainDate(1990, 1, 1).toZonedDateTime("UTC").epochNanoseconds,
  new Temporal.PlainDate(2000, 1, 1).toZonedDateTime("UTC").epochNanoseconds,
  new Temporal.PlainDate(2010, 1, 1).toZonedDateTime("UTC").epochNanoseconds,
  new Temporal.PlainDate(2020, 1, 1).toZonedDateTime("UTC").epochNanoseconds,
  new Temporal.PlainDate(2030, 1, 1).toZonedDateTime("UTC").epochNanoseconds,
];

for (const testCases of [asiaHistoricalTestCases,
                         africaHistoricalTestCases,
                         australasiaHistoricalTestCases,
                         europeHistoricalTestCases,
                         northAmericaHistoricalTestCases]) {
  for (let link of testCases) {
    const instanceLink = new Temporal.ZonedDateTime(0n, link);
    assert.sameValue(instanceLink.timeZoneId, link, `creating ZonedDateTime for ${link}`);
  }
}

for (const testCases of [asiaTestCases,
                         africaTestCases,
                         australasiaTestCases,
                         europeTestCases,
                         northAmericaTestCases,
                         otherTestCases]) {
  for (let [link, zone] of Object.entries(testCases)) {
    const instanceLink = new Temporal.ZonedDateTime(0n, link);
    assert.sameValue(instanceLink.timeZoneId, link, `creating ZonedDateTime for ${link}`);

    const instanceZone = new Temporal.ZonedDateTime(0n, zone);
    assert.sameValue(instanceLink.equals(instanceZone), true, `link=${link}, zone=${zone}`);

    assert.sameValue(
      instanceLink.offsetNanoseconds,
      instanceZone.offsetNanoseconds,
      `offsetNanoseconds; link=${link}, zone=${zone}`);

    for (let epochNs of epochNanoseconds) {
      assert.sameValue(
        new Temporal.ZonedDateTime(epochNs, link).offsetNanoseconds,
        new Temporal.ZonedDateTime(epochNs, zone).offsetNanoseconds,
        `link=${link}, zone=${zone}, epochNs=${epochNs}`
      );
    }
  }
}
