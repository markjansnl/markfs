extern crate fuse;
extern crate time;
extern crate libc;
extern crate rusqlite;
extern crate uuid;

use std::env;
use std::ffi::OsString;
use markfs::MarkFS;

mod markfs;
mod metadata;

fn main () {
    let args: Vec<OsString> = env::args_os().collect();

    if args.len() != 3 {
        println!("Usage: {:?} <local_path> <mountpoint>", &args[0]);
        ::std::process::exit(-1);
    }

    let local_path = &args[1];
    let mountpoint = &args[2];

    fuse::mount(MarkFS::new(local_path), mountpoint, &[]).unwrap();
}
