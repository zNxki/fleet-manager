use super::models::{nz, Document};
use super::{dberr, Db, R};
use rusqlite::params;

impl Db {
    pub fn list_documents(&self) -> R<Vec<Document>> {
        query_list!(
            self,
            "SELECT id,vehicle_id,driver_id,doc_type,number,issuer,issue_date,expiry_date,file_path,notes FROM documents ORDER BY expiry_date,id",
            |r| Ok(Document {
                id: r.get(0)?,
                vehicle_id: r.get::<_, Option<i64>>(1)?.unwrap_or(0),
                driver_id: r.get::<_, Option<i64>>(2)?.unwrap_or(0),
                doc_type: r.get(3)?,
                number: r.get(4)?,
                issuer: r.get(5)?,
                issue_date: r.get(6)?,
                expiry_date: r.get(7)?,
                file_path: r.get(8)?,
                notes: r.get(9)?,
            })
        )
    }

    pub fn save_document(&self, d: &Document) -> R<i64> {
        if (d.vehicle_id == 0) == (d.driver_id == 0) {
            return Err("A document must belong to exactly one vehicle or driver".to_string());
        }
        if d.expiry_date.is_empty() {
            return Err("The expiry date is required".to_string());
        }
        if !d.issue_date.is_empty() && d.expiry_date < d.issue_date {
            return Err("The expiry date cannot precede the issue date".to_string());
        }
        if d.id != 0 {
            self.conn
                .execute(
                    "UPDATE documents SET vehicle_id=?1,driver_id=?2,doc_type=?3,number=?4,issuer=?5,issue_date=?6,expiry_date=?7,file_path=?8,notes=?9 WHERE id=?10",
                    params![
                        nz(d.vehicle_id), nz(d.driver_id), d.doc_type, d.number, d.issuer,
                        d.issue_date, d.expiry_date, d.file_path, d.notes, d.id
                    ],
                )
                .map_err(dberr)?;
            Ok(d.id)
        } else {
            self.conn
                .execute(
                    "INSERT INTO documents(vehicle_id,driver_id,doc_type,number,issuer,issue_date,expiry_date,file_path,notes) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                    params![
                        nz(d.vehicle_id), nz(d.driver_id), d.doc_type, d.number, d.issuer,
                        d.issue_date, d.expiry_date, d.file_path, d.notes
                    ],
                )
                .map_err(dberr)?;
            Ok(self.conn.last_insert_rowid())
        }
    }

    pub fn delete_document(&self, id: i64) -> R<()> {
        self.conn
            .execute("DELETE FROM documents WHERE id=?1", params![id])
            .map_err(dberr)?;
        Ok(())
    }
}
