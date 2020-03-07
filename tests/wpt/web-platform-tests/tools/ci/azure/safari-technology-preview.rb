cask 'safari-technology-preview' do
  if MacOS.version <= :mojave
    version '102,061-84687-20200304-286a8fe8-5a00-41ba-9dba-2c42e047ffb1'
    sha256 '2e048fec4a470f5b5d9cffc359ba967bee02cf3cf121e5ea16812077adc5ba60'
  else
    version '102,061-84666-20200304-c2d165f2-2aec-41b2-86ef-0a9407608e2c'
    sha256 'ef9e9ea8990bb63936216fcdf93720b854366d158cad3daf5b812f841f7b5d99'
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
