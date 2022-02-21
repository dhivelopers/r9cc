text = """
assert 3 'a=3; return a;'
assert 8 'a=3; z=5; return a+z;'

assert 3 'a=3; return a;'
assert 8 'a=3; z=5; return a+z;'
assert 6 'a=b=3; return a+b;'
assert 3 'foo=3; return foo;'
assert 8 'foo123=3; bar=5; return foo123+bar;'

assert 1 'return 1; 2; 3;'
assert 2 '1; return 2; 3;'
assert 3 '1; 2; return 3;'
"""

# use this at project top directory

from pathlib import Path 

text = text.split('\n')

for i in range(len(text)):
    inp = text[i]
    inp = inp.lstrip("assert ")
    inp = inp.split(" ", 1)
    if inp == ['']:
        continue
    test_out = inp[0].strip("\"").strip("\'")
    test_in = inp[-1].strip("\"").strip("\'")
    print(test_in, test_out)
    test_name = input("Input test name\n> ")
    print(test_name)
    test_name = Path("./tests/testcases/").joinpath(test_name)
    while Path(test_name).exists():
        test_name = input("Input another test name\n> ")
        test_name = Path("./tests/testcases/").joinpath(test_name)
    test_name.mkdir()
    path_in = Path(test_name).joinpath('in')
    path_out = Path(test_name).joinpath('out')
    print(test_in)
    file_in = open(path_in, 'w')
    file_in.write(test_in)
    file_in.close()
    print(test_out)
    file_out = open(path_out, 'w')
    file_out.write(test_out)
    file_out.close()
