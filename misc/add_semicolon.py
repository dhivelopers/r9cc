import glob
from pathlib import Path
dirs = glob.glob("./tests/testcases/*")
print(dirs)
for dir in dirs:
    name = Path(dir).joinpath('in')
    file = open(name, 'a')
    file.write(";")
    file.close()
