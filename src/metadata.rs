use rusqlite::Connection;
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
}

#[derive(Debug, Clone)]
pub struct INode {
    pub ino: u64,
    pub id: String,
    pub parent: String,
    pub name: String,
    pub kind: INodeKind,
    pub size: u64,
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
                        nlink       INTEGER NOT NULL
                    )", &[]).unwrap();

        let root_guid = Uuid::new_v4().to_string();
        let root_name = "".to_string();

        conn.execute("INSERT INTO inode (ino, id, parent, name, kind, size, nlink)
                      VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6)",
                     &[&1, &root_guid, &root_name, &(INodeKind::Directory as i32), &0, &2]).unwrap();

        let hello_txt_guid = Uuid::new_v4().to_string();
        let hello_txt_name = "hello.txt".to_string();

        conn.execute("INSERT INTO inode (id, parent, name, kind, size, nlink)
                      VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                     &[&hello_txt_guid, &root_guid, &hello_txt_name, &(INodeKind::RegularFile as i32), &13, &1]).unwrap();

        Metadata {
            conn: conn
        }
    }

    pub fn get_by_ino(&self, ino: u64) -> Option<INode> {
        let mut stmt = self.conn.prepare("SELECT id, parent, name, kind, size, nlink FROM inode WHERE ino = ?1").unwrap();
        let mut inode_iter = stmt.query_map(&[&(ino as u32)], |row| {
            let size: i64 = row.get(4);

            INode {
                ino: ino,
                id: row.get(0),
                parent: row.get(1),
                name: row.get(2),
                kind: INodeKind::from_i32(row.get(3)).unwrap(),
                size: size as u64,
                nlink: row.get(5)
            }
        }).unwrap();

        Some(inode_iter.nth(0).unwrap().unwrap())
    }

    pub fn get_by_id(&self, id: String) -> Option<INode> {
        let mut stmt = self.conn.prepare("SELECT ino, parent, name, kind, size, nlink FROM inode WHERE id = ?1").unwrap();
        let mut inode_iter = stmt.query_map(&[&id], |row| {
            let ino: i64 = row.get(0);
            let size: i64 = row.get(4);

            INode {
                ino: ino as u64,
                id: id.clone(),
                parent: row.get(1),
                name: row.get(2),
                kind: INodeKind::from_i32(row.get(3)).unwrap(),
                size: size as u64,
                nlink: row.get(5)
            }
        }).unwrap();

        Some(inode_iter.nth(0).unwrap().unwrap())
    }

    pub fn lookup(&self, parent: String, name: String) -> Option<INode> {
        let mut stmt = self.conn.prepare("SELECT ino, id, kind, size, nlink FROM inode WHERE parent = ?1 AND name = ?2").unwrap();
        let mut inode_iter = stmt.query_map(&[&parent, &name], |row| {
            let ino: i64 = row.get(0);
            let size: i64 = row.get(3);

            INode {
                ino: ino as u64,
                id: row.get(1),
                parent: parent.clone(),
                name: name.clone(),
                kind: INodeKind::from_i32(row.get(2)).unwrap(),
                size: size as u64,
                nlink: row.get(4)
            }
        }).unwrap();

        Some(inode_iter.nth(0).unwrap().unwrap())
    }

    pub fn get_children(&self, parent: String) -> Vec<INode> {
        let mut stmt = self.conn.prepare("SELECT ino, id, parent, name, kind, size, nlink FROM inode WHERE parent = ?1 AND id <> ?1").unwrap();
        let inode_iter = stmt.query_map(&[&parent], |row| {
            let ino: i64 = row.get(0);
            let size: i64 = row.get(5);

            INode {
                ino: ino as u64,
                id: row.get(1),
                parent: row.get(2),
                name: row.get(3),
                kind: INodeKind::from_i32(row.get(4)).unwrap(),
                size: size as u64,
                nlink: row.get(6)
            }
        }).unwrap();

        let mut children = Vec::new();
        for child in inode_iter {
            children.push(child.unwrap());
        }
        children
    }
}
