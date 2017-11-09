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
    pub fn new() -> Metadata {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute("CREATE TABLE inode (
                        ino         INTEGER PRIMARY KEY,
                        id          TEXT NOT NULL,
                        parent      TEXT NOT NULL,
                        name        TEXT NOT NULL,
                        kind        INTEGER NOT NULL,
                        size        INTEGER NOT NULL,
                        atime       TEXT NOT NULL,
                        mtime       TEXT NOT NULL,
                        ctime       TEXT NOT NULL,
                        crtime      TEXT NOT NULL,
                        nlink       INTEGER NOT NULL
                    )", &[]).unwrap();

        let root_guid = Uuid::new_v4().to_string();
        let root_name = "".to_string();
        let create_time = time::get_time();

        conn.execute("INSERT INTO inode (ino, id, parent, name, kind, size, atime, mtime, ctime, crtime, nlink)
                      VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6, ?6, ?6, ?6, ?7)",
                     &[&1, &root_guid, &root_name, &(INodeKind::Directory as i32), &0, &create_time, &2]).unwrap();

        let hello_txt_guid = Uuid::new_v4().to_string();
        let hello_txt_name = "hello.txt".to_string();

        conn.execute("INSERT INTO inode (id, parent, name, kind, size, atime, mtime, ctime, crtime, nlink)
                      VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?6, ?6, ?7)",
                     &[&hello_txt_guid, &root_guid, &hello_txt_name, &(INodeKind::RegularFile as i32), &13, &create_time, &1]).unwrap();

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
        let mut stmt = self.conn.prepare(sql.as_str()).unwrap();
        let mut rows = stmt.query(params).unwrap();

        let mut inodes = Vec::new();
        while let Some(result_row) = rows.next() {
            let row = result_row.unwrap();
            let ino: i64 = row.get(0);
            let size: i64 = row.get(5);

            inodes.push(INode {
                ino: ino as u64,
                id: row.get(1),
                parent: row.get(2),
                name: row.get(3),
                kind: INodeKind::from_i32(row.get(4)).unwrap(),
                size: size as u64,
                atime: row.get(6),
                mtime: row.get(7),
                ctime: row.get(8),
                crtime: row.get(9),
                nlink: row.get(10)
            });
        }
        inodes
    }

}
