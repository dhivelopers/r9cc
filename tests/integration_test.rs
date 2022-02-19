use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

// read ./testcases/**/{in, out, (asm)}
// TODO: parallelly

fn remove_trailing_newline(mut s: String) -> String {
    if s.ends_with('\n') {
        s.pop();
    }
    s
}

#[test]
fn make_generated_test_sh() -> std::io::Result<()> {
    let template_prefix = r###"#!/bin/bash
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

# --- This is generated test ---

"###;
    let template_postfix = "\n\n# --- end of testcases ---\n\necho OK\n";
    let mut testcases = vec![];
    let dirs: Vec<PathBuf> = fs::read_dir(Path::new("tests/testcases"))?
        .map(|entry| {
            let entry = entry.unwrap();
            entry.path()
        })
        .collect();
    for dir in dirs {
        let test_in = dir.join("in");
        let test_out = dir.join("out");
        // let test_asm = dir.join("asm");
        let testcase = format!(
            "assert '{}' '{}'   # {}",
            remove_trailing_newline(fs::read_to_string(test_out)?),
            remove_trailing_newline(fs::read_to_string(test_in)?),
            dir.as_path().file_name().unwrap().to_str().unwrap()
        );
        testcases.push(testcase);
    }
    let mut file = File::create("generated_test.sh")?;
    file.write_all(
        format!(
            "{}{}{}",
            template_prefix,
            testcases.join("\n"),
            template_postfix
        )
        .as_bytes(),
    )?;
    Ok(())
}
