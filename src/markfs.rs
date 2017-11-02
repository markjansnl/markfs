use std::ffi::OsString;
use fuse::{Filesystem, Request, FileType, FileAttr, ReplyAttr, ReplyDirectory};
use time::Timespec;
use libc::ENOENT;
use metadata::Metadata;

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

const CREATE_TIME: Timespec = Timespec { sec: 1509654242, nsec: 0 };    // 2017-11-02 21:24:02

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
        match self.metadata.get_inode(ino) {
            Some(inode) => {
                let attr = FileAttr {
	                ino: ino,
    	            size: inode.size as u64,
        	        blocks: 0,
            	    atime: CREATE_TIME,
                	mtime: CREATE_TIME,
	                ctime: CREATE_TIME,
    	            crtime: CREATE_TIME,
        	        kind: match inode.kind {
            	        0 => FileType::Directory,
                	    _ => FileType::RegularFile
	                },
    	            perm: 0o755,
        	        nlink: inode.nlink,
            	    uid: 501,
                	gid: 20,
	                rdev: 0,
    	            flags: 0
        	    };

                reply.attr(&TTL, &attr);
            },
            None => {
                reply.error(ENOENT);
            }
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, mut reply: ReplyDirectory) {
        if ino == 1 {
            if offset == 0 {
                let inode = self.metadata.get_inode(ino).unwrap();
                let parent = self.metadata.get_inode(ino).unwrap();
                let hello_txt = self.metadata.get_inode(2).unwrap();

                reply.add(inode.ino, 0, FileType::Directory, ".");
                reply.add(parent.ino, 1, FileType::Directory, "..");
                reply.add(hello_txt.ino, 2, FileType::RegularFile, hello_txt.name);
            }
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }
}
