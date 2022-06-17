import os

entries = os.listdir('./level_src')
for entry in entries:
    if entry.endswith('.tmx'):
        os.system(f"tiled ./level_src/{entry} --embed-tilesets --export-map ./levels/{entry}")