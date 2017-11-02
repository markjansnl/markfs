use time::Timespec;
use rusqlite::Connection;
use uuid::Uuid;
use fuse::{FileType, FileAttr};

const CREATE_TIME: Timespec = Timespec { sec: 1509654242, nsec: 0 };    // 2017-11-02 21:24:02

struct INode {
    ino: u32,
    id: String,
    parent: String,
    name: String,
    kind: FileType
}

pub struct Metadata {
    conn: Connection
}

impl Metadata {
    pub fn new() -> Metadata {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute("CREATE TABLE inode (
                        ino         INTEGER PRIMARY KEY,
                        id          TEXT NOT NULL,
                        parent      TEXT NOT NULL,
                        name        TEXT NOT NULL,
                        kind        INTEGER NOT NULL,
                        size        INTEGER NOT NULL,
                        nlink       INTEGER NOT NULL
                    )", &[]).unwrap();

        let root_guid = Uuid::new_v4().to_string();
        let root_name = "".to_string();

        conn.execute("INSERT INTO inode (ino, id, parent, name, kind, size, nlink)
                      VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6)",
                     &[&1, &root_guid, &root_name, &0, &0, &2]).unwrap();

        let hello_txt_guid = Uuid::new_v4().to_string();
        let hello_txt_name = "hello_txt".to_string();

        conn.execute("INSERT INTO inode (id, parent, name, kind, size, nlink)
                      VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                     &[&hello_txt_guid, &root_guid, &hello_txt_name, &1, &13, &1]).unwrap();

        Metadata {
            conn: conn
        }
    }

    pub fn get_attr(&self, ino: u64) -> Option<FileAttr> {
        let mut stmt = self.conn.prepare("SELECT kind, size, nlink FROM inode WHERE ino = ?1").unwrap();
        let mut inode_iter = stmt.query_map(&[&(ino as u32)], |row| {
            let size: i64 = row.get(1);

            FileAttr {
                ino: ino,
                size: size as u64,
                blocks: 0,
                atime: CREATE_TIME,
                mtime: CREATE_TIME,
                ctime: CREATE_TIME,
                crtime: CREATE_TIME,
                kind: match row.get(0) {
                    0 => FileType::Directory,
                    _ => FileType::RegularFile
                },
                perm: 0o755,
                nlink: row.get(2),
                uid: 501,
                gid: 20,
                rdev: 0,
                flags: 0
            }
        }).unwrap();

        Some(inode_iter.nth(0).unwrap().unwrap())
    }

}
