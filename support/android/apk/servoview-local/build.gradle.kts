configurations.maybeCreate("default")

val servoViewLocal: String? by gradle.extra

if (servoViewLocal != null) {
    val aar = File(servoViewLocal!!)
    if (aar.exists()) {
        artifacts.add("default", aar)
    } else {
        throw GradleException("Failed to find ServoView AAR at: $servoViewLocal")
    }
} else {
    throw GradleException("Local ServoView AAR path not defined")
}
