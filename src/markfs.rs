use std::ffi::{OsStr, OsString};
use fuse::{Filesystem, Request, FileType, FileAttr, ReplyEntry, ReplyAttr, ReplyDirectory, ReplyData};
use time::Timespec;
use libc::ENOENT;
use metadata::{Metadata, INode, INodeKind};

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

const CREATE_TIME: Timespec = Timespec { sec: 1509654242, nsec: 0 };    // 2017-11-02 21:24:02

const HELLO_TXT_CONTENT: &'static str = "Hello World!\n";

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

    fn inode_kind_to_file_type(&self, kind: &INodeKind) -> FileType {
        match *kind {
            INodeKind::Directory   => FileType::Directory,
            INodeKind::RegularFile => FileType::RegularFile
        }
    }

    fn inode_to_fileattr(&self, inode: INode) -> FileAttr {
        FileAttr {
            ino: inode.ino,
            size: inode.size,
            blocks: 0,
            atime: CREATE_TIME,
            mtime: CREATE_TIME,
            ctime: CREATE_TIME,
            crtime: CREATE_TIME,
            kind: self.inode_kind_to_file_type(&inode.kind),
            perm: 0o775,
            nlink: inode.nlink,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0
        }
    }
}

impl Filesystem for MarkFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let parent_inode = self.metadata.get_by_ino(parent).unwrap();
        let name_string = match name.to_str() {
            Some(name_slice) => {
                name_slice.to_string()
            },
            None => {
                // UTF-8 conversion error
                reply.error(ENOENT);
                return;
            }
        };

        match self.metadata.lookup(&parent_inode.id, &name_string) {
            Some(inode) => {
                reply.entry(&TTL, &self.inode_to_fileattr(inode), 0);
            },
            None => {
                reply.error(ENOENT);
            }
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.metadata.get_by_ino(ino) {
            Some(inode) => {
                reply.attr(&TTL, &self.inode_to_fileattr(inode));
            },
            None => {
                reply.error(ENOENT);
            }
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, mut reply: ReplyDirectory) {
        match self.metadata.get_by_ino(ino) {
            Some(inode) => {
                if inode.kind == INodeKind::Directory {
                    if offset == 0 {
                        let parent = self.metadata.get_by_id(&inode.parent).unwrap();

                        reply.add(inode.ino, 0, FileType::Directory, ".");
                        reply.add(parent.ino, 1, FileType::Directory, "..");

                        let mut index = 2;
                        for child in self.metadata.get_children(&inode.parent) {
                            reply.add(child.ino, index, self.inode_kind_to_file_type(&child.kind), child.name);
                            index += 1;
                        }
                    }
                    reply.ok();
                } else {
                    reply.error(ENOENT);
                }
            },
            None => {
                reply.error(ENOENT);
            }
        }
    }

    fn read (&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, _size: u32, reply: ReplyData) {
        println!("Used target: {:?}", self.target);
        if ino == 2 {
            reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
        } else {
            reply.error(ENOENT);
        }
    }
}
