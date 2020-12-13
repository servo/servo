cask "safari-technology-preview" do
  if MacOS.version <= :catalina
    version "117,001-86197-20201209-2fd0fe89-0c37-412d-99e2-4e288519e886"
    sha256 "739630b43a8f021cc246b2c8f610b759e82a6336830d34dac1f388763e7cceea"
  else
    version "117,001-86222-20201209-8021be92-32b3-403f-b9a7-630ab8e91afb"
    sha256 "2dc42342e1e3fdd95086dfbd3a8588f1e2d181fbd4f8bb279994be0c2e57ff6f"
  end

  url "https://secure-appldnld.apple.com/STP/#{version.after_comma}/SafariTechnologyPreview.dmg"
  appcast "https://developer.apple.com/safari/download/"
  name "Safari Technology Preview"
  homepage "https://developer.apple.com/safari/download/"

  auto_updates true
  depends_on macos: ">= :catalina"

  pkg "Safari Technology Preview.pkg"

  uninstall delete: "/Applications/Safari Technology Preview.app"

  zap trash: [
    "~/Library/Application Support/com.apple.sharedfilelist/com.apple.LSSharedFileList.ApplicationRecentDocuments/com.apple.safaritechnologypreview.sfl*",
    "~/Library/Caches/com.apple.SafariTechnologyPreview",
    "~/Library/Preferences/com.apple.SafariTechnologyPreview.plist",
    "~/Library/SafariTechnologyPreview",
    "~/Library/Saved Application State/com.apple.SafariTechnologyPreview.savedState",
    "~/Library/SyncedPreferences/com.apple.SafariTechnologyPreview-com.apple.Safari.UserRequests.plist",
    "~/Library/SyncedPreferences/com.apple.SafariTechnologyPreview-com.apple.Safari.WebFeedSubscriptions.plist",
    "~/Library/SyncedPreferences/com.apple.SafariTechnologyPreview.plist",
    "~/Library/WebKit/com.apple.SafariTechnologyPreview",
  ]
end
