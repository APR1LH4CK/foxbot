import os
from pathlib import Path

foxes_dir = Path(r"C:\Users\april\Documents\Projects\foxbot\images\foxes")
jpg_files = list(foxes_dir.glob("*.jpg"))

file_info = []
for file_path in jpg_files:
    stat = file_path.stat()
    parts = file_path.name.split('-')
    author_name = f"{parts[0]}-{parts[1]}" if len(parts) > 1 else parts[0]
    file_info.append((file_path, stat.st_ctime, author_name))

file_info.sort(key=lambda x: x[1])

for i, (file_path, _, author_name) in enumerate(file_info, 1):
    new_name = f"{author_name}-{i}.jpg"
    new_path = file_path.parent / new_name
    
    if not new_path.exists():
        file_path.rename(new_path)
        print(f"{file_path.name} -> {new_name}")
    else:
        print(f"skip: {new_name} already exists")