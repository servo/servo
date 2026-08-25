// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The depth of nested function calls reaches 32
es5id: 13.2.1_A1_T1
description: Creating function calls 32 elements depth
---*/

(function(){
    (function(){
        (function(){
            (function(){
                (function(){
                    (function(){
                        (function(){
                            (function(){
                                (function(){
                                    (function(){
                                        (function(){
                                            (function(){
                                                (function(){
                                                    (function(){
                                                        (function(){
                                                            (function(){
                                                                (function(){
                                                                    (function(){
                                                                        (function(){
                                                                            (function(){
                                                                                (function(){
                                                                                    (function(){
                                                                                        (function(){
                                                                                            (function(){
                                                                                                (function(){
                                                                                                    (function(){
                                                                                                        (function(){
                                                                                                            (function(){
                                                                                                                (function(){
                                                                                                                    (function(){
                                                                                                                        (function(){
                                                                                                                            (function(){})()
                                                                                                                        })()
                                                                                                                    })()
                                                                                                                })()
                                                                                                            })()
                                                                                                        })()
                                                                                                    })()
                                                                                                })()
                                                                                            })()
                                                                                        })()
                                                                                    })()
                                                                                })()
                                                                            })()
                                                                        })()
                                                                    })()
                                                                })()
                                                            })()
                                                        })()
                                                    })()
                                                })()
                                            })()
                                        })()
                                    })()
                                })()
                            })()
                        })()
                    })()
                })()
            })()
        })()
    })()
})()
