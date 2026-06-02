import java.text.SimpleDateFormat
import java.util.Date

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

val generatedVersionCode = versionCodeString.toInt()