IF NOT EXIST .\image_results mkdir .\image_results

REM PNG to compressed DDS (BC2) = DXT3
AMDCompressCLI -fd BC2 .\images_todo\foo.png .\image_results\foo.dds