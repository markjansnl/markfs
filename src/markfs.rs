use std::ffi::{OsStr, OsString};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::{SeekFrom};
use std::io::prelude::*;
use fuse::{Filesystem, Request, FileType, FileAttr, ReplyEntry, ReplyAttr, ReplyDirectory, ReplyOpen, ReplyEmpty, ReplyData};
use time::Timespec;
use libc::ENOENT;
use metadata::{Metadata, INode, INodeKind};

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

enum FileHandle {
    Local(File)
}

pub struct MarkFS {
    local_path: OsString,
    metadata: Metadata,
    open_fh: HashMap<u64, FileHandle>,
    last_fh: u64
}

impl MarkFS {
    pub fn new(local_path: &OsString) -> MarkFS {
        MarkFS {
            local_path: local_path.clone(),
            metadata: Metadata::new(&local_path),
            open_fh: HashMap::new(),
            last_fh: 0
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
            atime: inode.atime,
            mtime: inode.mtime,
            ctime: inode.ctime,
            crtime: inode.crtime,
            kind: self.inode_kind_to_file_type(&inode.kind),
            perm: 0o775,
            nlink: inode.nlink,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0
        }
    }

    fn get_path(&self, inode: &INode, path_buf: &mut PathBuf) {
        if inode.ino == 1 {
            path_buf.push(&self.local_path);
        } else {
            let parent_inode = self.metadata.get_by_id(&inode.parent).unwrap();
            self.get_path(&parent_inode, path_buf);
            path_buf.push(&inode.name);

            if inode.kind.is_regular_file() {
                path_buf.push(&inode.current_version);
            }
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
                if inode.kind.is_directory() {
                    if offset == 0 {
                        let parent = self.metadata.get_by_id(&inode.parent).unwrap();

                        reply.add(inode.ino, 0, FileType::Directory, ".");
                        reply.add(parent.ino, 1, FileType::Directory, "..");

                        let mut index = 2;
                        for child in self.metadata.get_children(&inode.id) {
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

    fn open(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        match self.metadata.get_by_ino(_ino) {
            Some(inode) => {
                if inode.kind.is_regular_file() {
                    let mut path_buf = PathBuf::new();
                    self.get_path(&inode, &mut path_buf);

                    let file = OpenOptions::new().read(true).open(path_buf).unwrap();
                    let file_handle = FileHandle::Local(file);
                    self.last_fh += 1;
                    self.open_fh.insert(self.last_fh, file_handle);

                    reply.opened(self.last_fh, _flags);
                } else {
                    reply.error(ENOENT);
                }
            },
            None => {
                reply.error(ENOENT);
            }
        }
    }

    fn release(&mut self, _req: &Request, _ino: u64, _fh: u64, _flags: u32, _lock_owner: u64, _flush: bool, reply: ReplyEmpty) {
        match self.open_fh.remove(&_fh) {
            Some(FileHandle::Local(_file)) => {
                reply.ok();
            },
            None => {
                reply.error(ENOENT);
            }
        }
    }

    fn read (&mut self, _req: &Request, _ino: u64, _fh: u64, offset: u64, _size: u32, reply: ReplyData) {
        match self.open_fh.get_mut(&_fh) {
            Some(&mut FileHandle::Local(ref mut file)) => {
                file.seek(SeekFrom::Start(offset)).unwrap();

                let mut data = Vec::<u8>::with_capacity(_size as usize);
                unsafe { data.set_len(_size as usize) };

                match file.read(&mut data) {
                    Ok(n) => {
                        data.truncate(n);
                        reply.data(data.as_slice());
                    },
                    Err(_e) => {
                        reply.error(ENOENT);
                    }
                }
            },
            None => {
                reply.error(ENOENT);
            }
        }
    }
}
