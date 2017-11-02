use std::ffi::OsString;
use fuse::{Filesystem, Request, ReplyAttr, ReplyDirectory};
use time::Timespec;
use libc::ENOENT;
use metadata::Metadata;

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

pub struct MarkFS {
    target: OsString,
    metadata: Metadata
}

impl MarkFS {
    pub fn new(target: OsString) -> MarkFS {
        MarkFS {
            target: target,
            metadata: Metadata::new()
        }
    }
}

impl Filesystem for MarkFS {
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.metadata.get_attr(ino) {
            Some(attr) => { reply.attr(&TTL, &attr); },
            None       => { reply.error(ENOENT); }
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, mut reply: ReplyDirectory) {
        let root = self.metadata.get_attr(1).unwrap();
        println!("Root node: {:?}", root);

        let hello_txt = self.metadata.get_attr(2).unwrap();
        println!("Hello node: {:?}", hello_txt);

        if ino == 1 {
            if offset == 0 {
                reply.add(root.ino, 0, root.kind, ".");
                reply.add(root.ino, 1, root.kind, ".");
                reply.add(hello_txt.ino, 2, hello_txt.kind, "hello.txt");
            }
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }
}
