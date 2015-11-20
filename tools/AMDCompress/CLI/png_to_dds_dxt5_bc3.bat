IF NOT EXIST .\image_results mkdir .\image_results

REM Example  2b: PNG to compressed DDS (BC3) = DXT5
AMDCompressCLI -fd BC3 .\images_todo\foo.png .\image_results\foo.dds