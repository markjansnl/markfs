use std::ffi::OsString;
use std::path::Path;
use rusqlite::Connection;
use rusqlite::types::ToSql;
use time;
use time::Timespec;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum INodeKind {
    Directory = 0,
    RegularFile,
}

impl INodeKind {
    pub fn from_i32(i: i32) -> Option<INodeKind> {
        match i {
            0 => Some(INodeKind::Directory),
            1 => Some(INodeKind::RegularFile),
            _ => None
        }
    }

    pub fn is_directory(&self) -> bool {
        *self == INodeKind::Directory
    }

    pub fn is_regular_file(&self) -> bool {
        *self == INodeKind::RegularFile
    }
}

#[derive(Debug, Clone)]
pub struct INode {
    pub ino: u64,
    pub id: String,
    pub parent: String,
    pub name: String,
    pub kind: INodeKind,
    pub size: u64,
    pub atime: Timespec,
    pub mtime: Timespec,
    pub ctime: Timespec,
    pub crtime: Timespec,
    pub nlink: u32
}

pub struct Metadata {
    conn: Connection
}

impl Metadata {
    pub fn new(local_path: &OsString) -> Metadata {
        let path_buf = Path::new(local_path).join("metadata.sqlite");
        let conn = Connection::open(path_buf.as_path()).unwrap();

        let create_table = conn.execute("
            CREATE TABLE inode (
                ino             INTEGER PRIMARY KEY,
                id              TEXT NOT NULL,
                parent          TEXT NOT NULL,
                name            TEXT NOT NULL,
                kind            INTEGER NOT NULL,
                atime           TEXT NOT NULL,
                mtime           TEXT NOT NULL,
                ctime           TEXT NOT NULL,
                crtime          TEXT NOT NULL,
                nlink           INTEGER NOT NULL,
                current_version TEXT NOT NULL
            )", &[]);

        if create_table.is_ok() {
            let root_guid = Uuid::new_v4().to_string();
            let root_name = "";
            let create_time = time::get_time();

            conn.execute("INSERT INTO inode (ino, id, parent, name, kind, atime, mtime, ctime, crtime, nlink, current_version)
                          VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?5, ?5, ?5, ?6, ?7)",
                         &[&1, &root_guid, &root_name, &(INodeKind::Directory as i32), &create_time, &2, &root_name]).unwrap();

            let hello_txt_guid = Uuid::new_v4().to_string();
            let hello_txt_name = "hello.txt";
            let version = "1";
            let source_version = "";
            let hash = "";

            conn.execute("INSERT INTO inode (id, parent, name, kind, atime, mtime, ctime, crtime, nlink, current_version)
                          VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?5, ?5, ?6, ?7)",
                         &[&hello_txt_guid, &root_guid, &hello_txt_name, &(INodeKind::RegularFile as i32), &create_time, &1, &version]).unwrap();

            conn.execute("
                CREATE TABLE file_version (
                    id              TEXT NOT NULL,
                    version         TEXT NOT NULL,
                    source_version  TEXT NOT NULL,
                    size            INTEGER NOT NULL,
                    hash            TEXT NOT NULL
                )", &[]).unwrap();


            conn.execute("INSERT INTO file_version (id, version, source_version, size, hash)
                          VALUES (?1, ?2, ?3, ?4, ?5)",
                         &[&hello_txt_guid, &version, &source_version, &13, &hash]).unwrap();
        }

        Metadata {
            conn: conn
        }
    }

    pub fn get_by_ino(&self, ino: u64) -> Option<INode> {
        self.query_inode("ino = ?1", &[&(ino as u32)]).pop()
    }

    pub fn get_by_id(&self, id: &String) -> Option<INode> {
        self.query_inode("id = ?1", &[&id.as_str()]).pop()
    }

    pub fn lookup(&self, parent: &String, name: &String) -> Option<INode> {
        self.query_inode("parent = ?1 AND name = ?2", &[&parent.as_str(), &name.as_str()]).pop()
    }

    pub fn get_children(&self, parent: &String) -> Vec<INode> {
        self.query_inode("parent = ?1 AND id <> ?1", &[&parent.as_str()])
    }

    fn query_inode(&self, where_clause: &str, params: &[&ToSql]) -> Vec<INode> {
        let sql = format!("SELECT * FROM inode WHERE {}", where_clause);
        let mut inode_stmt = self.conn.prepare(sql.as_str()).unwrap();
        let mut inode_rows = inode_stmt.query(params).unwrap();

        let mut inodes = Vec::new();
        while let Some(result_row) = inode_rows.next() {
            let row = result_row.unwrap();
            let ino: i64 = row.get(0);
            let id: String = row.get(1);
            let mut size: i64 = 0;
            let kind: INodeKind = INodeKind::from_i32(row.get(4)).unwrap();
            let current_version: String = row.get(10);

            if kind.is_regular_file() {
                let mut file_version_stmt = self.conn.prepare("SELECT size FROM file_version WHERE id = ?1 AND version = ?2").unwrap();
                file_version_stmt.query_map(&[&id, &current_version], |file_version_row| {
                    size = file_version_row.get(0);
                }).unwrap().next();
            }

            inodes.push(INode {
                ino: ino as u64,
                id,
                parent: row.get(2),
                name: row.get(3),
                kind,
                size: size as u64,
                atime: row.get(5),
                mtime: row.get(6),
                ctime: row.get(7),
                crtime: row.get(8),
                nlink: row.get(9),
            });
        }
        inodes
    }

}
