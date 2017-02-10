extern crate pkg_config;
extern crate bindgen;

use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use pkg_config::Error;

const HEADER: &'static str = "./src/libr.h";
const BINDINGS: &'static str = "src/bindings.rs";
const RLIBS: [&'static str; 20] = [
    "r_core", "r_bin", "r_search",
    "r_cons", "r_io", "r_util",
    "r_bp", "r_magic", "r_reg",
    "r_config", "r_asm", "r_flag",
    "r_anal", "r_parse", "r_fs",
    "r_debug", "r_hash", "r_lang",
    "r_syscall", "r_socket" ];

fn main() {
  let mut include_paths:Vec<std::path::PathBuf> = Vec::new();
  let mut link_paths:Vec<std::path::PathBuf> = Vec::new();
  let mut exit_code = 0;
  // leverage pkg-config to find libr include path
  for lib in RLIBS.iter() {
    match pkg_config::probe_library(lib) {
      Ok(l) => {
        for p in l.include_paths.iter() { include_paths.push(p.clone()); }
        for p in l.link_paths.iter() { link_paths.push(p.clone()); }
      },
      Err(Error::Failure { output, .. }) => {
        std::io::stderr().write(output.stderr.as_slice()).unwrap();
        exit_code = output.status.code().unwrap();
      },
      Err(e) => panic!("Unhandled error: {:?}", e),
    };
    if exit_code!=0 { std::process::exit(exit_code); }
  }
  include_paths.sort();
  include_paths.dedup();
  // found libr, continue
  let mut builder = bindgen::Builder::default()
    .no_unstable_rust()
    .clang_arg("-DHAVE_LIB_SSL=0"); // only one type r_socket::r_socket_t
  // radare uses <r_types>, we can't use cargo:include...
  // as clang doesn't import these correctly, use -I/path/to/.h instead.
  builder = include_paths.iter()
    .fold(builder,|b,i| b.clang_arg(format!{"-I{}", i.to_str().unwrap()}));
  builder = builder.header(HEADER);
  // hide rust internal types
  let types = vec!{"INFINITY", "NAN"};
  builder = types.iter()
    .fold(builder,|b,t| b.hide_type(t));
  // TODO siginfo_t, sigaction, in6_addr, RMagic types need ffi safety
  // store final bindings
  let bindings = builder.generate().expect("Unable to generate bindings").to_string();
  let out_path = PathBuf::from(BINDINGS);
  File::create(&out_path).expect("Problem creating output file")
    .write_all(&bindings.into_bytes())
    .expect("Unable to write output");
}
