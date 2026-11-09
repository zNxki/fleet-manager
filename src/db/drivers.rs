use super::models::Driver;
use super::{dberr, Db, R};
use rusqlite::params;

impl Db {
    pub fn list_drivers(&self) -> R<Vec<Driver>> {
        query_list!(
            self,
            "SELECT id,first_name,last_name,tax_code,birth_date,phone,email,address,license_number,license_category,license_expiry,hire_date,job_title,employment_type,active,notes FROM drivers ORDER BY last_name,first_name",
            |r| Ok(Driver {
                id: r.get(0)?,
                first_name: r.get(1)?,
                last_name: r.get(2)?,
                tax_code: r.get(3)?,
                birth_date: r.get(4)?,
                phone: r.get(5)?,
                email: r.get(6)?,
                address: r.get(7)?,
                license_number: r.get(8)?,
                license_category: r.get(9)?,
                license_expiry: r.get(10)?,
                hire_date: r.get(11)?,
                job_title: r.get(12)?,
                employment_type: r.get(13)?,
                active: r.get::<_, i64>(14)? != 0,
                notes: r.get(15)?,
            })
        )
    }

    pub fn has_open_driver_assignment(&self, did: i64) -> R<bool> {
        let n: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM assignments WHERE driver_id=?1 AND end_date=''",
                params![did],
                |r| r.get(0),
            )
            .map_err(dberr)?;
        Ok(n > 0)
    }

    pub fn save_driver(&self, d: &Driver) -> R<i64> {
        if d.id != 0 && !d.active && self.has_open_driver_assignment(d.id)? {
            return Err("The driver has an open assignment: register the return first".to_string());
        }
        if d.id != 0 {
            self.conn
                .execute(
                    "UPDATE drivers SET first_name=?1,last_name=?2,tax_code=?3,birth_date=?4,phone=?5,email=?6,address=?7,license_number=?8,license_category=?9,license_expiry=?10,hire_date=?11,job_title=?12,employment_type=?13,active=?14,notes=?15 WHERE id=?16",
                    params![
                        d.first_name, d.last_name, d.tax_code, d.birth_date, d.phone, d.email,
                        d.address, d.license_number, d.license_category, d.license_expiry,
                        d.hire_date, d.job_title, d.employment_type, d.active as i64, d.notes, d.id
                    ],
                )
                .map_err(dberr)?;
            Ok(d.id)
        } else {
            self.conn
                .execute(
                    "INSERT INTO drivers(first_name,last_name,tax_code,birth_date,phone,email,address,license_number,license_category,license_expiry,hire_date,job_title,employment_type,active,notes) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
                    params![
                        d.first_name, d.last_name, d.tax_code, d.birth_date, d.phone, d.email,
                        d.address, d.license_number, d.license_category, d.license_expiry,
                        d.hire_date, d.job_title, d.employment_type, d.active as i64, d.notes
                    ],
                )
                .map_err(dberr)?;
            Ok(self.conn.last_insert_rowid())
        }
    }

    pub fn delete_driver(&self, id: i64) -> R<()> {
        if self.has_open_driver_assignment(id)? {
            return Err("The driver has an open assignment: register the return first".to_string());
        }
        self.conn
            .execute("DELETE FROM drivers WHERE id=?1", params![id])
            .map_err(dberr)?;
        Ok(())
    }
}
