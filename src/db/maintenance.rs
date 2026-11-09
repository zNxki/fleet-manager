use super::models::Maintenance;
use super::{dberr, Db, R};
use crate::util::format as f;
use rusqlite::params;

impl Db {
    pub fn list_maintenance(&self) -> R<Vec<Maintenance>> {
        query_list!(
            self,
            "SELECT id,vehicle_id,kind,description,provider,scheduled_date,completed_date,cost,mileage,status,next_due_km,next_due_date FROM maintenance ORDER BY COALESCE(NULLIF(completed_date,''),scheduled_date) DESC,id DESC",
            |r| Ok(Maintenance {
                id: r.get(0)?,
                vehicle_id: r.get(1)?,
                kind: r.get(2)?,
                description: r.get(3)?,
                provider: r.get(4)?,
                scheduled_date: r.get(5)?,
                completed_date: r.get(6)?,
                cost: r.get(7)?,
                mileage: r.get(8)?,
                status: r.get(9)?,
                next_due_km: r.get(10)?,
                next_due_date: r.get(11)?,
            })
        )
    }

    pub fn save_maintenance(&self, m: &Maintenance) -> R<i64> {
        if m.vehicle_id == 0 {
            return Err("Select a vehicle".to_string());
        }
        if m.description.trim().is_empty() {
            return Err("Description is required".to_string());
        }
        let today = f::today_iso();
        match m.status.as_str() {
            "Completed" => {
                if m.completed_date.is_empty() {
                    return Err("The completion date is required for completed records".to_string());
                }
                if m.completed_date > today {
                    return Err("The completion date cannot be in the future".to_string());
                }
            }
            "In progress" => {
                if self.has_open_assignment(m.vehicle_id)? {
                    return Err(
                        "Register the vehicle return before sending it to maintenance".to_string(),
                    );
                }
                let others: i64 = self
                    .conn
                    .query_row(
                        "SELECT COUNT(*) FROM maintenance WHERE vehicle_id=?1 AND status='In progress' AND id<>?2",
                        params![m.vehicle_id, m.id],
                        |r| r.get(0),
                    )
                    .map_err(dberr)?;
                if others > 0 {
                    return Err("The vehicle already has a maintenance in progress".to_string());
                }
                if !m.completed_date.is_empty() {
                    return Err("Only completed records have a completion date".to_string());
                }
            }
            _ => {
                if !m.completed_date.is_empty() {
                    return Err("Only completed records have a completion date".to_string());
                }
            }
        }
        if m.status != "Completed" && m.scheduled_date.is_empty() {
            return Err("The scheduled date is required".to_string());
        }
        if m.id != 0 {
            self.conn
                .execute(
                    "UPDATE maintenance SET vehicle_id=?1,kind=?2,description=?3,provider=?4,scheduled_date=?5,completed_date=?6,cost=?7,mileage=?8,status=?9,next_due_km=?10,next_due_date=?11 WHERE id=?12",
                    params![
                        m.vehicle_id, m.kind, m.description, m.provider, m.scheduled_date,
                        m.completed_date, m.cost, m.mileage, m.status, m.next_due_km,
                        m.next_due_date, m.id
                    ],
                )
                .map_err(dberr)?;
        } else {
            self.conn
                .execute(
                    "INSERT INTO maintenance(vehicle_id,kind,description,provider,scheduled_date,completed_date,cost,mileage,status,next_due_km,next_due_date) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                    params![
                        m.vehicle_id, m.kind, m.description, m.provider, m.scheduled_date,
                        m.completed_date, m.cost, m.mileage, m.status, m.next_due_km,
                        m.next_due_date
                    ],
                )
                .map_err(dberr)?;
        }
        let id = if m.id != 0 {
            m.id
        } else {
            self.conn.last_insert_rowid()
        };
        if m.mileage > 0 {
            self.bump_mileage(m.vehicle_id, m.mileage)?;
        }
        Ok(id)
    }

    pub fn delete_maintenance(&self, id: i64) -> R<()> {
        self.conn
            .execute("DELETE FROM maintenance WHERE id=?1", params![id])
            .map_err(dberr)?;
        Ok(())
    }
}
