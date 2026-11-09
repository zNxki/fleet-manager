use super::models::Assignment;
use super::{dberr, Db, R};
use crate::util::format as f;
use rusqlite::params;

const FAR: &str = "9999-12-31";

impl Db {
    pub fn list_assignments(&self) -> R<Vec<Assignment>> {
        query_list!(
            self,
            "SELECT id,vehicle_id,driver_id,start_date,end_date,start_km,end_km,notes FROM assignments ORDER BY start_date DESC,id DESC",
            |r| Ok(Assignment {
                id: r.get(0)?,
                vehicle_id: r.get(1)?,
                driver_id: r.get(2)?,
                start_date: r.get(3)?,
                end_date: r.get(4)?,
                start_km: r.get(5)?,
                end_km: r.get(6)?,
                notes: r.get(7)?,
            })
        )
    }

    pub fn has_open_assignment(&self, vid: i64) -> R<bool> {
        let n: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM assignments WHERE vehicle_id=?1 AND end_date=''",
                params![vid],
                |r| r.get(0),
            )
            .map_err(dberr)?;
        Ok(n > 0)
    }

    fn overlaps(&self, col: &str, key: i64, ignore: i64, s: &str, e: &str) -> R<bool> {
        let sql = format!(
            "SELECT start_date,end_date FROM assignments WHERE {}=?1 AND id<>?2",
            col
        );
        let mut st = self.conn.prepare(&sql).map_err(dberr)?;
        let rows = st
            .query_map(params![key, ignore], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(dberr)?;
        let list = rows.collect::<Result<Vec<_>, _>>().map_err(dberr)?;
        let e_eff = if e.is_empty() { FAR } else { e };
        for (os, oe) in list {
            let oe_eff = if oe.is_empty() { FAR.to_string() } else { oe };
            if os.as_str() <= e_eff && s <= oe_eff.as_str() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn validate_assignment(&self, a: &Assignment) -> R<()> {
        let v = self
            .list_vehicles()?
            .into_iter()
            .find(|v| v.id == a.vehicle_id)
            .ok_or("Vehicle not found")?;
        let d = self
            .list_drivers()?
            .into_iter()
            .find(|d| d.id == a.driver_id)
            .ok_or("Driver not found")?;
        if v.base_status == "Retired" {
            return Err("Retired vehicles cannot be assigned".to_string());
        }
        if !d.active {
            return Err("The driver is not active".to_string());
        }
        let today = f::today_iso();
        if a.start_date > today {
            return Err("The start date cannot be in the future".to_string());
        }
        if d.license_expiry.is_empty() || d.license_expiry < a.start_date {
            return Err("The driving license is expired on the assignment start date".to_string());
        }
        if !a.end_date.is_empty() {
            if a.end_date < a.start_date {
                return Err("The return date cannot precede the start date".to_string());
            }
            if a.end_date > today {
                return Err("The return date cannot be in the future".to_string());
            }
            match a.end_km {
                Some(k) if k >= a.start_km => {}
                _ => {
                    return Err(
                        "The return mileage cannot be lower than the start mileage".to_string()
                    )
                }
            }
        } else {
            let in_progress: i64 = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM maintenance WHERE vehicle_id=?1 AND status='In progress'",
                    params![a.vehicle_id],
                    |r| r.get(0),
                )
                .map_err(dberr)?;
            if in_progress > 0 {
                return Err("The vehicle is currently in maintenance".to_string());
            }
            if a.id == 0 && a.start_km < v.mileage {
                return Err(format!(
                    "The start mileage cannot be lower than the vehicle mileage ({})",
                    v.mileage
                ));
            }
        }
        if self.overlaps("vehicle_id", a.vehicle_id, a.id, &a.start_date, &a.end_date)? {
            return Err("The vehicle already has an overlapping assignment".to_string());
        }
        if self.overlaps("driver_id", a.driver_id, a.id, &a.start_date, &a.end_date)? {
            return Err("The driver already has an overlapping assignment".to_string());
        }
        Ok(())
    }

    pub fn create_assignment(&self, a: &Assignment) -> R<i64> {
        self.validate_assignment(a)?;
        self.conn
            .execute(
                "INSERT INTO assignments(vehicle_id,driver_id,start_date,end_date,start_km,end_km,notes) VALUES(?1,?2,?3,?4,?5,?6,?7)",
                params![a.vehicle_id, a.driver_id, a.start_date, a.end_date, a.start_km, a.end_km, a.notes],
            )
            .map_err(dberr)?;
        let id = self.conn.last_insert_rowid();
        self.bump_mileage(a.vehicle_id, a.end_km.unwrap_or(a.start_km))?;
        Ok(id)
    }

    pub fn close_assignment(&self, id: i64, end_date: &str, end_km: i64) -> R<()> {
        let mut a = self
            .list_assignments()?
            .into_iter()
            .find(|x| x.id == id)
            .ok_or("Assignment not found")?;
        if !a.end_date.is_empty() {
            return Err("The assignment is already closed".to_string());
        }
        a.end_date = end_date.to_string();
        a.end_km = Some(end_km);
        self.validate_assignment(&a)?;
        self.conn
            .execute(
                "UPDATE assignments SET end_date=?1,end_km=?2 WHERE id=?3",
                params![a.end_date, end_km, id],
            )
            .map_err(dberr)?;
        self.bump_mileage(a.vehicle_id, end_km)
    }

    pub fn delete_assignment(&self, id: i64) -> R<()> {
        self.conn
            .execute("DELETE FROM assignments WHERE id=?1", params![id])
            .map_err(dberr)?;
        Ok(())
    }
}
