IF NOT EXIST .\image_results mkdir .\image_results

REM PNG to compressed DDS (BC1) = DXT1
AMDCompressCLI -fd BC1 .\images_todo\foo.png .\image_results\foo.dds