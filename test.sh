#!/bin/bash
assert() {
  expected="$1"
  input="$2"

  # use release binary
  ./target/release/r9cc "$input" > tmp.s
  cc -o tmp tmp.s
  ./tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

# build release binary
cargo build --release

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

echo OK
