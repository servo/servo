cask 'safari-technology-preview' do
  if MacOS.version <= :mojave
    version '103,061-90754-20200325-37467264-9c34-454c-be4a-1cb87e93c62c'
    sha256 'f2175a2ca69152b6c1067d47b56d464b0ba0c71323a76137b488a8088a25f44c'
  else
    version '103,061-90752-20200325-cc5192c4-619a-45b4-83b5-70a1bed8c912'
    sha256 '846f2c9e7ebcc293f01ea36c8e1184e2d1bfb985eb3a590fc7c730c40e10a4b6'
  end

  url "https://secure-appldnld.apple.com/STP/#{version.after_comma}/SafariTechnologyPreview.dmg"
  appcast 'https://developer.apple.com/safari/download/'
  name 'Safari Technology Preview'
  homepage 'https://developer.apple.com/safari/download/'

  auto_updates true
  depends_on macos: '>= :mojave'

  pkg 'Safari Technology Preview.pkg'

  uninstall delete: '/Applications/Safari Technology Preview.app'

  zap trash: [
               '~/Library/Application Support/com.apple.sharedfilelist/com.apple.LSSharedFileList.ApplicationRecentDocuments/com.apple.safaritechnologypreview.sfl*',
               '~/Library/Caches/com.apple.SafariTechnologyPreview',
               '~/Library/Preferences/com.apple.SafariTechnologyPreview.plist',
               '~/Library/SafariTechnologyPreview',
               '~/Library/Saved Application State/com.apple.SafariTechnologyPreview.savedState',
               '~/Library/SyncedPreferences/com.apple.SafariTechnologyPreview-com.apple.Safari.UserRequests.plist',
               '~/Library/SyncedPreferences/com.apple.SafariTechnologyPreview-com.apple.Safari.WebFeedSubscriptions.plist',
               '~/Library/SyncedPreferences/com.apple.SafariTechnologyPreview.plist',
               '~/Library/WebKit/com.apple.SafariTechnologyPreview',
             ]
end
