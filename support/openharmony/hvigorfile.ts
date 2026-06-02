import { appTasks, OhosAppContext, OhosPluginId } from '@ohos/hvigor-ohos-plugin';
import { getNode } from '@ohos/hvigor'
import * as fs from 'fs';
import * as path from 'path';

const rootNode = getNode(__filename);
rootNode.afterNodeEvaluate(node => {
    const appContext = node.getContext(OhosPluginId.OHOS_APP_PLUGIN) as OhosAppContext;
    const buildProfileOpt = appContext.getBuildProfileOpt();
    const signingConfigsPath = process.env["SERVO_OHOS_SIGNING_CONFIG"];
    if (signingConfigsPath) {
        if (!fs.existsSync(signingConfigsPath)) {
            console.error("File referenced by SERVO_OHOS_SIGNING_CONFIG does not exist!");
            return;
        }
        const basePath = path.dirname(signingConfigsPath);
        const signingConfigs = JSON.parse(fs.readFileSync(signingConfigsPath));
        for (const config of signingConfigs) {
            config.material.certpath = path.resolve(basePath, config.material.certpath);
            config.material.profile = path.resolve(basePath, config.material.profile);
            config.material.storeFile = path.resolve(basePath, config.material.storeFile);
        }
        buildProfileOpt['app']['signingConfigs'] = signingConfigs;
    } else {
        console.log('Signing config not found in enviroment. hvigor will fallback to build-profile.json5.')
    }
    // Set the obj object back to the context object to enable the build process and results.
    appContext.setBuildProfileOpt(buildProfileOpt);
})

export default {
    system: appTasks,  /* Built-in plugin of Hvigor. It cannot be modified. */
    plugins:[]         /* Custom plugin to extend the functionality of Hvigor. */
}
