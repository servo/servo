import java.util.regex.Pattern

plugins {
    id("com.android.application")
}

android {
    compileSdk = 33
    buildToolsVersion = "34.0.0"

    namespace = "org.servo.servoshell"

    layout.buildDirectory = File(rootDir.absolutePath, "/../../../target/android/gradle/servoapp")

    defaultConfig {
        applicationId = "org.servo.servoshell"
        minSdk = 30
        targetSdk = 33
        versionCode = generatedVersionCode
        versionName = "0.0.1" // TODO: Parse Servo"s TOML and add git SHA.
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    // Share all of that with servoview
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

    sourceSets {
        named("main") {
            java.srcDirs("src/main/java")
        }
    }

    val signingKeyInfo = getSigningKeyInfo()

    if (signingKeyInfo != null) {
        signingConfigs {
            register("release") {
                storeFile = signingKeyInfo["storeFile"] as File
                storePassword = signingKeyInfo["storePassword"] as String
                keyAlias = signingKeyInfo["keyAlias"] as String
                keyPassword = signingKeyInfo["keyPassword"] as String
            }
        }
    }

    buildTypes {
        debug {
        }

        release {
            signingConfig =
                signingConfigs.getByName(if (signingKeyInfo != null) "release" else "debug")
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android.txt"), "proguard-rules.pro")
        }

        // Custom build types

        val debug = getByName("debug")
        val release = getByName("release")


        register("armv7Debug") {
            initWith(debug)
            ndk {
                abiFilters.add(getNDKAbi("armv7"))
            }
        }
        register("armv7Release") {
            initWith(release)
            ndk {
                abiFilters.add(getNDKAbi("armv7"))
            }
        }
        register("arm64Debug") {
            initWith(debug)
            ndk {
                abiFilters.add(getNDKAbi("arm64"))
            }
        }
        register("arm64Release") {
            initWith(release)
            ndk {
                abiFilters.add(getNDKAbi("arm64"))
            }
        }
        register("x86Debug") {
            initWith(debug)
            ndk {
                abiFilters.add(getNDKAbi("x86"))
            }
        }
        register("x86Release") {
            initWith(release)
            ndk {
                abiFilters.add(getNDKAbi("x86"))
            }
        }
        register("x64Debug") {
            initWith(debug)
            ndk {
                abiFilters.add(getNDKAbi("x64"))
            }
        }
        register("x64Release") {
            initWith(release)
            ndk {
                abiFilters.add(getNDKAbi("x64"))
            }
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

    project.afterEvaluate {
        android.applicationVariants.forEach { variant ->
            val pattern = Pattern.compile("^[\\w\\d]+([A-Z][\\w\\d]+)(Debug|Release)")
            val matcher = pattern.matcher(variant.name)
            if (!matcher.find()) {
                throw GradleException("Invalid variant name for output: " + variant.name)
            }
            val arch = matcher.group(1)
            val debug = variant.name.contains("Debug")
            val finalFolder = getTargetDir(debug, arch)
            val finalFile = File(finalFolder, "servoapp.apk")
            variant.outputs.forEach { output ->
                val copyAndRenameAPKTask =
                    project.task<Copy>("copyAndRename${variant.name.capitalize()}APK") {
                        from(output.outputFile.parent)
                        into(finalFolder)
                        include(output.outputFile.name)
                        rename(output.outputFile.name, finalFile.name)
                    }
                variant.assembleProvider.get().finalizedBy(copyAndRenameAPKTask)
            }
        }
    }
}

dependencies {
    if (findProject(":servoview-local") != null) {
        implementation(project(":servoview-local"))
    } else {
        implementation(project(":servoview"))
    }

    implementation("androidx.appcompat:appcompat:1.6.1")
    implementation("com.google.android.material:material:1.9.0")
    implementation("androidx.constraintlayout:constraintlayout:2.1.3")
}
