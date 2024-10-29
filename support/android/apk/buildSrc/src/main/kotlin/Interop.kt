import org.gradle.api.GradleException
import org.gradle.api.Project
import java.io.File
import java.util.Locale
import java.util.Properties

/*
Some functions are extensions to the Project class, as to allow access to its public members.
 */

fun Project.getTargetDir(debug: Boolean, arch: String): String {
    val basePath = project.rootDir.parentFile.parentFile.parentFile.absolutePath
    return basePath + "/target/android/" + getSubTargetDir(debug, arch)
}

fun Project.getNativeTargetDir(debug: Boolean, arch: String): String {
    val basePath = project.rootDir.parentFile.parentFile.parentFile.absolutePath
    return basePath + "/target/" + getSubTargetDir(debug, arch)
}

fun getSubTargetDir(debug: Boolean, arch: String): String {
    return getRustTarget(arch) + "/" + if (debug) "debug" else "release"
}

fun Project.getJniLibsPath(debug: Boolean, arch: String): String =
    getTargetDir(debug, arch) + "/jniLibs"

fun getRustTarget(arch: String): String {
    return when (arch.lowercase(Locale.getDefault())) {
        "armv7" -> "armv7-linux-androideabi"
        "arm64" -> "aarch64-linux-android"
        "x86" -> "i686-linux-android"
        "x64" -> "x86_64-linux-android"
        else -> throw GradleException("Invalid target architecture $arch")
    }
}

fun getNDKAbi(arch: String): String {
    return when (arch.lowercase(Locale.getDefault())) {
        "armv7" -> "armeabi-v7a"
        "arm64" -> "arm64-v8a"
        "x86" -> "x86"
        "x64" -> "x86_64"
        else -> throw GradleException("Invalid target architecture $arch")
    }
}

fun Project.getNdkDir(): String {
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
    return ndkDir.absolutePath
}