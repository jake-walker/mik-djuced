# MIK DJUCED

This is a work in progress tool to copy data from [Mixed In Key 11](https://mixedinkey.com/learn-more/) to [DJUCED](https://www.djuced.com/).

## Compatibility

This has been only tested on MacOS. I don't have a Windows version of Mixed In Key so cannot test it for that. I expect the Mixed In Key database is different on Windows, so this is unlikely to work without some modifications.

## Usage

Songs must first be imported into DJUCED as normal so they populate the database. Next, use Mixed In Key as normal with the newly imported songs. Finally, close both programs and run this script, which will copy metadata and hotcues from the Mixed In Key database and update the DJUCED database.

## Todo & Known Issues

- Existing hot cues for any songs will be removed and overwritten, even if no hot cues are added to MIK.
