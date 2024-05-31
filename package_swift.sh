set -eu

zip -r IrohLib.xcframework.zip Iroh.xcframework/*
swift package compute-checksum IrohLib.xcframework.zip


# TODO(b5): finish upload to github
# require GH_TOKEN is set in the environment
# if [ -z "$GH_TOKEN" ]; then
#     echo "GH_TOKEN is not set"
#     exit 1
# fi

# attach the zip to the latest release
# latest_release=$(curl -s -H "Authorization: token $GH_TOKEN" https://api.github.com/repos/n0-computer/iroh-ffi/releases/latest | jq -r .tag_name)
# echo "Latest release: $latest_release"
# upload_url=$(curl -s -H "Authorization: token $GH_TOKEN" https://api.github.com/repos/n0-computer/iroh-ffi/releases/latest | jq -r .upload_url | sed 's/{?name,label}//')
# echo "Upload URL: $upload_url"
# curl -s -H "Authorization: token $GH_TOKEN" -H "Content-Type: application/zip" --data-binary @IrohLib.xcframework.zip "$upload_url?name=IrohLib.xcframework.zip"