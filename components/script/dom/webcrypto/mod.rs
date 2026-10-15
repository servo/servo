/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_webcrypto::traits::WebCryptoGlobalTrait;

use crate::dom::GlobalScope;
use crate::tasks::task::TaskOnce;

pub(crate) mod crypto {
    pub(crate) type Crypto = script_webcrypto::crypto::Crypto<crate::DomTypeHolder>;
}
pub(crate) mod cryptokey {
    pub(crate) type CryptoKey = script_webcrypto::cryptokey::CryptoKey<crate::DomTypeHolder>;
}
pub(crate) mod subtlecrypto {
    pub(crate) type SubtleCrypto =
        script_webcrypto::subtlecrypto::SubtleCrypto<crate::DomTypeHolder>;
}

impl WebCryptoGlobalTrait<crate::DomTypeHolder> for GlobalScope {
    fn queue_crypto_task_source(&self, task: impl TaskOnce + 'static) {
        self.task_manager().crypto_task_source().queue(task);
    }

    fn queue_dom_manipulation_task_source(&self, task: impl TaskOnce + 'static) {
        self.task_manager()
            .dom_manipulation_task_source()
            .queue(task);
    }
}
