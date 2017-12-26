use std::ffi::{OsStr, OsString};
use std::collections::HashMap;
use std::path::PathBuf;
use fuse::{Filesystem, Request, FileType, FileAttr, ReplyEntry, ReplyAttr, ReplyDirectory, ReplyOpen, ReplyEmpty, ReplyData};
use time::Timespec;
use libc::{ENOENT, ENOSYS};
use metadata::{Metadata, INode, INodeKind};

use local::LocalFileHandle;
use local::LocalFileOperations;

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

pub trait FileHandle {
    fn read(&mut self, offset: i64, size: u32) -> Result<Vec<u8>, ()>;
}

pub struct MarkFS {
    local_path: OsString,
    metadata: Metadata,
    open_fh: HashMap<u64, Box<FileHandle>>,
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
            uid: 501,
            gid: 20,
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
        }
    }
}

impl Filesystem for MarkFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let parent_inode = match self.metadata.get_by_ino(parent) {
            Some(inode) => inode,
            None => {
                reply.error(ENOENT);
                return;
            }
        };
        let name_string = match name.to_str() {
            Some(slice) => slice.to_string(),
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        match self.metadata.lookup(&parent_inode, &name_string) {
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

    fn mkdir(&mut self, _req: &Request, _parent: u64, _name: &OsStr, _mode: u32, reply: ReplyEntry) {
        let parent_inode = match self.metadata.get_by_ino(_parent) {
            Some(inode) => inode,
            None => {
                reply.error(ENOENT);
                return;
            }
        };
        let name_string = match _name.to_str() {
            Some(slice) => slice.to_string(),
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        match self.metadata.create_dir(&parent_inode, &name_string) {
            Ok(inode) => {
                let mut path_buf = PathBuf::new();
                self.get_path(&inode, &mut path_buf);

                match LocalFileOperations::create_dir(&path_buf.as_path()) {
                    Ok(_) => {
                        reply.entry(&TTL, &self.inode_to_fileattr(inode), 0);
                    },
                    Err(_) => {
                        reply.error(ENOSYS);
                    }
                }
            },
            Err(_) => {
                reply.error(ENOENT);
            }
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        match self.metadata.get_by_ino(ino) {
            Some(inode) => {
                if inode.kind.is_directory() {
                    if offset == 0 {
                        let parent = self.metadata.get_by_id(&inode.parent).unwrap();

                        reply.add(inode.ino, 0, FileType::Directory, ".");
                        reply.add(parent.ino, 1, FileType::Directory, "..");

                        let mut index = 2;
                        for child in self.metadata.get_children(&inode) {
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

                    let file_handle = LocalFileHandle::new(path_buf.as_path(), _flags as i32);
                    self.last_fh += 1;
                    self.open_fh.insert(self.last_fh, Box::new(file_handle));

                    reply.opened(self.last_fh, _flags);
                } else {
                    reply.error(ENOSYS);
                }
            },
            None => {
                reply.error(ENOENT);
            }
        }
    }

    fn release(&mut self, _req: &Request, _ino: u64, _fh: u64, _flags: u32, _lock_owner: u64, _flush: bool, reply: ReplyEmpty) {
        match self.open_fh.remove(&_fh) {
            Some(_) => {
                reply.ok();
            },
            None => {
                reply.error(ENOSYS);
            }
        }
    }

    fn read (&mut self, _req: &Request, _ino: u64, _fh: u64, offset: i64, _size: u32, reply: ReplyData) {
        match self.open_fh.get_mut(&_fh) {
            Some(ref mut file_handle) => {
                match file_handle.read(offset, _size) {
                    Ok(data) => {
                        reply.data(data.as_slice());
                    },
                    Err(_e) => {
                        reply.error(ENOSYS);
                    }
                }
            },
            None => {
                reply.error(ENOSYS);
            }
        }
    }

    fn rename(&mut self, _req: &Request, _parent: u64, _name: &OsStr, _newparent: u64, _newname: &OsStr, reply: ReplyEmpty) {
        let parent_inode = match self.metadata.get_by_ino(_parent) {
            Some(inode) => inode,
            None => {
                reply.error(ENOENT);
                return;
            }
        };
        let new_parent_inode = match self.metadata.get_by_ino(_newparent) {
            Some(inode) => inode,
            None => {
                reply.error(ENOENT);
                return;
            }
        };
        let name_string = match _name.to_str() {
            Some(slice) => slice.to_string(),
            None => {
                reply.error(ENOENT);
                return;
            }
        };
        let new_name_string = match _newname.to_str() {
            Some(slice) => slice.to_string(),
            None => {
                reply.error(ENOSYS);
                return;
            }
        };

        match self.metadata.lookup(&parent_inode, &name_string) {
            Some(old_inode) => {
                match self.metadata.rename(&old_inode, &new_parent_inode, &new_name_string) {
                    Ok(new_inode) => {
                        let mut path_buf_old = PathBuf::new();
                        self.get_path(&old_inode, &mut path_buf_old);

                        let mut path_buf_new = PathBuf::new();
                        self.get_path(&new_inode, &mut path_buf_new);

                        match LocalFileOperations::rename(&path_buf_old.as_path(), &path_buf_new.as_path()) {
                            Ok(()) => {
                                reply.ok();
                            },
                            Err(_) => {
                                reply.error(ENOSYS);
                            }
                        }
                    },
                    Err(_) => {
                        reply.error(ENOSYS);
                    }
                }
            },
            None => {
                reply.error(ENOENT);
            }
        }
    }
}
