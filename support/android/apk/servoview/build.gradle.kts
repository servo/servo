import java.io.FileFilter
import java.util.regex.Pattern

plugins {
    id("com.android.library")
}

android {
    compileSdk = 33
    buildToolsVersion = "34.0.0"

    namespace = "org.servo.servoview"

    layout.buildDirectory = File(rootDir.absolutePath, "/../../../target/android/gradle/servoview")

    ndkPath = getNdkDir()

    defaultConfig {
        minSdk = 30
        lint.targetSdk = 33
        defaultConfig.versionCode = generatedVersionCode
        defaultConfig.versionName = "0.0.1" // TODO: Parse Servo"s TOML and add git SHA.
    }

    compileOptions {
        compileOptions.incremental = false
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    flavorDimensions.add("default")

    productFlavors {
        register("basic") {
        }
    }

    splits {
        density {
            isEnable = false
        }
        abi {
            isEnable = false
        }
    }


    buildTypes {
        // Default debug and release build types are used as templates
        debug {
            isJniDebuggable = true
        }

        release {
            signingConfig =
                signingConfigs.getByName("debug") // Change this to sign with a production key
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android.txt"), "proguard-rules.pro")
        }

        val debug = getByName("debug")
        val release = getByName("release")

        // Custom build types
        register("armv7Debug") {
            initWith(debug)
        }
        register("armv7Release") {
            initWith(release)
        }
        register("arm64Debug") {
            initWith(debug)
        }
        register("arm64Release") {
            initWith(release)
        }
        register("x86Debug") {
            initWith(debug)
        }
        register("x86Release") {
            initWith(release)
        }
        register("x64Debug") {
            initWith(debug)
        }
        register("x64Release") {
            initWith(release)
        }
    }

    sourceSets {
        named("main") {
        }
        named("armv7Debug") {
            jniLibs.srcDirs(getJniLibsPath(true, "armv7"))
        }
        named("armv7Release") {
            jniLibs.srcDirs(getJniLibsPath(false, "armv7"))
        }
        named("arm64Debug") {
            jniLibs.srcDirs(getJniLibsPath(true, "arm64"))
        }
        named("arm64Release") {
            jniLibs.srcDirs(getJniLibsPath(false, "arm64"))
        }
        named("x86Debug") {
            jniLibs.srcDirs(getJniLibsPath(true, "x86"))
        }
        named("x86Release") {
            jniLibs.srcDirs(getJniLibsPath(false, "x86"))
        }
        named("x64Debug") {
            jniLibs.srcDirs(getJniLibsPath(true, "x64"))
        }
        named("x64Release") {
            jniLibs.srcDirs(getJniLibsPath(false, "x64"))
        }
    }

    // Ignore default "debug" and "release" build types
    androidComponents {
        beforeVariants {
            if (it.buildType == "release" || it.buildType == "debug") {
                it.enable = false
            }
        }
    }

    // Call our custom NDK Build task using flavor parameters.
    // This step is needed because the Android Gradle Plugin system"s
    // integration with native C/C++ shared objects (based on the
    // `android.externalNativeBuild` dsl object) assumes that we
    // actually execute compiler commands to produced the shared
    // objects. We already have the libsimpleservo.so produced by rustc.
    // We could simply copy the .so to the `sourceSet.jniLibs` folder
    // to make AGP bundle it with the APK, but this doesn"t copy the STL
    // (libc++_shared.so) as well. So we use ndk-build as a glorified
    // `cp` command to copy the libsimpleservo.so from target/<arch>
    // to target/android and crucially also include libc++_shared.so
    // as well.
    //
    // FIXME(mukilan): According to the AGP docs, we should not be
    // relying on task names used by the plugin system to hook into
    // the build process, but instead we should use officially supported
    // extension points such as `androidComponents.beforeVariants`
    project.afterEvaluate {
        // we filter entries first in order to abstract to a new list
        // as to prevent concurrent modification exceptions due to creating a new task
        // while iterating
        tasks.mapNotNull { compileTask -> // mapNotNull acts as our filter, null results are dropped
            // This matches the task `mergeBasicArmv7DebugJniLibFolders`.
            val pattern = Pattern.compile("^merge[A-Z]\\w+([A-Z]\\w+)(Debug|Release)JniLibFolders")
            val matcher = pattern.matcher(compileTask.name)
            if (matcher.find())
                compileTask to matcher.group(1)
            else null
        }.forEach { (compileTask, arch) ->
            val ndkBuildTask = tasks.create<Exec>("ndkbuild" + compileTask.name) {
                val debug = compileTask.name.contains("Debug")
                commandLine(
                    getNdkDir() + "/ndk-build",
                    "APP_BUILD_SCRIPT=../jni/Android.mk",
                    "NDK_APPLICATION_MK=../jni/Application.mk",
                    "NDK_LIBS_OUT=" + getJniLibsPath(debug, arch),
                    "NDK_DEBUG=" + if (debug) "1" else "0",
                    "APP_ABI=" + getNDKAbi(arch),
                    "NDK_LOG=1",
                    "SERVO_TARGET_DIR=" + getNativeTargetDir(debug, arch)
                )
            }

            compileTask.dependsOn(ndkBuildTask)
        }

        android.libraryVariants.forEach { variant ->
            val pattern = Pattern.compile("^[\\w\\d]+([A-Z][\\w\\d]+)(Debug|Release)")
            val matcher = pattern.matcher(variant.name)
            if (!matcher.find()) {
                throw GradleException("Invalid variant name for output: " + variant.name)
            }
            val arch = matcher.group(1)
            val debug = variant.name.contains("Debug")
            val finalFolder = getTargetDir(debug, arch)
            val finalFile = File(finalFolder, "servoview.aar")
            variant.outputs.forEach { output ->
                val copyAndRenameAARTask =
                    project.task<Copy>("copyAndRename${variant.name.capitalize()}AAR") {
                        from(output.outputFile.parent)
                        into(finalFolder)
                        include(output.outputFile.name)
                        rename(output.outputFile.name, finalFile.name)
                    }
                variant.assembleProvider.get().finalizedBy(copyAndRenameAARTask)
            }
        }
    }
}

dependencies {

    //Dependency list
    val deps = listOf(ServoDependency("blurdroid.jar", "blurdroid"))
    // Iterate all build types and dependencies
    // For each dependency call the proper implementation command and set the correct dependency path
    val list = listOf("armv7", "arm64", "x86", "x64")
    for (arch in list) {
        for (debug in listOf(true, false)) {
            val basePath = getTargetDir(debug, arch) + "/build"
            val cmd = arch + (if (debug) "Debug" else "Release") + "Implementation"

            for (dep in deps) {
                val path = findDependencyPath(basePath, dep.fileName, dep.folderFilter)
                if (path.isNotBlank())
                    cmd(files(path)) // this is how custom flavors are called.
            }
        }
    }
}

// folderFilter can be used to improve search performance
fun findDependencyPath(basePath: String, filename: String, folderFilter: String?): String {
    var path = File(basePath)
    if (!path.exists()) {
        return ""
    }


    if (folderFilter != null) {
        path.listFiles(FileFilter { it.isDirectory })!!.forEach {
            if (it.name.contains(folderFilter)) {
                path = File(it.absolutePath)
            }
        }
    }

    var result = ""

    for (file in path.walkTopDown()) {
        if (file.isFile && file.name == filename) {
            result = file.absolutePath
            break // no more walking
        }
    }

    return result
}

data class ServoDependency(val fileName: String, val folderFilter: String? = null)
