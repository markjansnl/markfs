use time::Timespec;
use rusqlite::Connection;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct INode {
    pub ino: u64,
    pub id: String,
    pub parent: String,
    pub name: String,
    pub kind: u32,
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

    pub fn get_inode(&self, ino: u64) -> Option<INode> {
        let mut stmt = self.conn.prepare("SELECT id, parent, name, kind, size, nlink FROM inode WHERE ino = ?1").unwrap();
        let mut inode_iter = stmt.query_map(&[&(ino as u32)], |row| {
            let size: i64 = row.get(4);

            INode {
                ino: ino,
                id: row.get(0),
                parent: row.get(1),
                name: row.get(2),
                kind: row.get(3),
                size: size as u64,
                nlink: row.get(5)
            }
        }).unwrap();

        Some(inode_iter.nth(0).unwrap().unwrap())
    }

}
