text = """
assert 0 0
assert 42 42
assert 21 "5+20-4"
assert 103 "4 + 100 - 1"
assert 102 "   4   + 100 - 2   "
assert 21 "1+2 * 10"
assert 30 "(1+2 ) * 10"
assert 47 "5+6*7"
assert 15 "5*(9-6)"
assert 4 "(3+5)/2"
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
    test_out = inp[0].strip("\"")
    test_in = inp[-1].strip("\"")
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
