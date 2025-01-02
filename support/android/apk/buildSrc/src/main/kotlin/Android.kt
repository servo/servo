import java.io.File

fun getSigningKeyInfo(): Map<String, Any>? {
    val storeFilePath = System.getenv("APK_SIGNING_KEY_STORE_PATH")
    return if (storeFilePath != null) {
        mapOf(
            "storeFile" to File(storeFilePath),
            "storePassword" to System.getenv("APK_SIGNING_KEY_STORE_PASS"),
            "keyAlias" to System.getenv("APK_SIGNING_KEY_ALIAS"),
            "keyPassword" to System.getenv("APK_SIGNING_KEY_PASS"),
        )
    } else {
        null
    }
}