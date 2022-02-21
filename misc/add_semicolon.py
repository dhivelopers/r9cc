import glob
from pathlib import Path
dirs = glob.glob("./tests/testcases/*")
print(dirs)
for dir in dirs:
    name = Path(dir).joinpath('in')
    file = open(name, 'r')
    content = file.read()
    file.close()

    arr = content.split(";")
    last = "return " + arr[-2].lstrip('\n') + ";"
    if len(arr) > 2:
        arr.pop() # ''
        arr.pop() # some;
        arr = [a.lstrip('\n') for a in arr]
        arr.append(last)
        content = ";\n".join(arr)
    else:
        content = last
    print(content)
    file = open(name, 'w')
    file.write(content)
    file.close()
