/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;
use std::{env, fs};

use chrono::{TimeZone, Utc};

fn main() {
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    // Parsing as suggested on <https://reproducible-builds.org/docs/source-date-epoch/>
    let now = match env::var("SOURCE_DATE_EPOCH") {
        Ok(val) => Utc
            .timestamp_opt(
                val.parse::<i64>()
                    .expect("SOURCE_DATE_EPOCH should be a valid integer"),
                0,
            )
            .unwrap(),
        Err(_) => Utc::now(),
    };

    let build_id = now.format("%Y%m%d%H%M%S").to_string();

    let path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("build_id.rs");
    // The build ID is used in Firefox devtools, `getDateFromBuildID` function:
    // <https://searchfox.org/firefox-main/rev/be31b3948198286e39a9855e414823cb17b6e94c/devtools/client/shared/remote-debugging/version-checker.js#21-24>
    // The expected format is: yyyyMMddHHmmss.
    // The date is than later used to check devtools compatibility:
    // <https://searchfox.org/firefox-main/rev/be31b3948198286e39a9855e414823cb17b6e94c/devtools/client/shared/remote-debugging/version-checker.js#133-139>
    fs::write(path, format!("const BUILD_ID: &str = \"{build_id}\";")).unwrap();
}
