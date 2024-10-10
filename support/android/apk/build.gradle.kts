import java.util.Date
import java.util.Properties
import java.text.SimpleDateFormat
import java.util.Locale

// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
    id("com.android.application") version "8.7.0" apply false
    id("com.android.library") version "8.7.0" apply false
}


// Utility methods
val getTargetDir = { debug: Boolean, arch: String ->
    val basePath = project.rootDir.parentFile.parentFile.parentFile.absolutePath
    basePath + "/target/android/" + getSubTargetDir(debug, arch)
}
ext.set(
    "getTargetDir",
    KotlinClosure2(getTargetDir)
)

ext.set(
    "getNativeTargetDir",
    KotlinClosure2({ debug: Boolean, arch: String ->
        val basePath = project.rootDir.parentFile.parentFile.parentFile.absolutePath
        basePath + "/target/" + getSubTargetDir(debug, arch)
    })
)

val getSubTargetDir = { debug: Boolean, arch: String ->
    getRustTarget(arch) + "/" + if (debug) "debug" else "release"
}

ext.set(
    "getSubTargetDir",
    KotlinClosure2(getSubTargetDir)
)

ext.set(
    "getJniLibsPath",
    KotlinClosure2({ debug: Boolean, arch: String ->
        getTargetDir(debug, arch) + "/jniLibs"
    })
)

val getRustTarget = { arch: String ->
    when (arch.lowercase(Locale.getDefault())) {
        "armv7" -> "armv7-linux-androideabi"
        "arm64" -> "aarch64-linux-android"
        "x86" -> "i686-linux-android"
        "x64" -> "x86_64-linux-android"
        else -> throw GradleException("Invalid target architecture $arch")
    }
}

ext.set(
    "getRustTarget",
    KotlinClosure1(getRustTarget)
)

ext.set(
    "getNDKAbi",
    KotlinClosure1<String, String>({
        when (this.lowercase(Locale.getDefault())) {
            "armv7" -> "armeabi-v7a"
            "arm64" -> "arm64-v8a"
            "x86" -> "x86"
            "x64" -> "x86_64"
            else -> throw GradleException("Invalid target architecture $this")
        }

    })
)

ext.set(
    "getNdkDir",
    KotlinClosure0({
        // Read environment variable used in rust build system
        var ndkRoot = System.getenv("ANDROID_NDK_ROOT")
        if (ndkRoot == null) {
            // Fallback to ndkDir in local.properties
            val rootDir = project.rootDir
            val localProperties = File(rootDir, "local.properties")
            val properties = Properties()
            localProperties.inputStream().use { instr ->
                properties.load(instr)
            }

            ndkRoot = properties.getProperty("ndk.dir")
        }

        val ndkDir = if (ndkRoot != null) File(ndkRoot) else null
        if (ndkDir == null || !ndkDir.exists()) {
            throw GradleException(
                "Please set a valid ANDROID_NDK_ROOT environment variable " +
                        "or ndk.dir path in local.properties file"
            )
        }
        ndkDir.absolutePath
    })
)

ext.set(
    "getSigningKeyInfo",
    KotlinClosure0<Map<String, Any>>({
        val storeFilePath = System.getenv("APK_SIGNING_KEY_STORE_PATH")
        if (storeFilePath != null) {
            mapOf(
                "storeFile" to File(storeFilePath),
                "storePassword" to System.getenv("APK_SIGNING_KEY_STORE_PASS"),
                "keyAlias" to System.getenv("APK_SIGNING_KEY_ALIAS"),
                "keyPassword" to System.getenv("APK_SIGNING_KEY_PASS"),
            )
        } else {
            null
        }
    })
)

// Generate unique version code based on the build date and time to support nightly
// builds.
//
// The version scheme is currently: yyyyMMddXX
// where
// yyyy is the 4 digit year (e.g 2024)
// MMdd is the 2 digit month followed by 2 digit day (e.g 0915 for September 15)
// XX is currently hardcoded to 00, but can be used to distinguish multiple builds within
// the same day.
//
// TODO: check if this interferes with caching of local builds and add option to use
// a static version.
val today = Date()
val versionCodeString: String = SimpleDateFormat("yyyyMMdd00").format(today)
extra.set("generatedVersionCode", versionCodeString.toInt())