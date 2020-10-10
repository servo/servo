cask "safari-technology-preview" do
  if MacOS.version <= :catalina
    version "114,001-54431-20201007-6899e5c7-457d-484b-bf38-fce2fcc967da"
    sha256 "1803c0affc0e7b28d945e122e99ed5741c19b9f2a949b473133adf916023db5b"
  else
    version "114,001-53239-20201007-7bed156d-ab68-4db2-8ff0-3fa2e23fa2c0"
    sha256 "c93ff3b5d485fce80b86f9656ffba3f5b967b189776cd20aea16a4c681419414"
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
