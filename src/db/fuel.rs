use super::models::{nz, Expense, Fuel};
use super::{dberr, Db, R};
use crate::util::format as f;
use rusqlite::params;

impl Db {
    pub fn list_fuel(&self) -> R<Vec<Fuel>> {
        query_list!(
            self,
            "SELECT id,vehicle_id,driver_id,date,liters,price,mileage,station,full_tank FROM fuel ORDER BY date DESC,id DESC",
            |r| Ok(Fuel {
                id: r.get(0)?,
                vehicle_id: r.get(1)?,
                driver_id: r.get::<_, Option<i64>>(2)?.unwrap_or(0),
                date: r.get(3)?,
                liters: r.get(4)?,
                price: r.get(5)?,
                mileage: r.get(6)?,
                station: r.get(7)?,
                full_tank: r.get::<_, i64>(8)? != 0,
            })
        )
    }

    pub fn save_fuel(&self, x: &Fuel) -> R<i64> {
        if x.vehicle_id == 0 {
            return Err("Select a vehicle".to_string());
        }
        if x.liters <= 0.0 || x.price <= 0.0 {
            return Err("Liters and price must be greater than zero".to_string());
        }
        if x.date > f::today_iso() {
            return Err("The refueling date cannot be in the future".to_string());
        }
        let bad: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM fuel WHERE vehicle_id=?1 AND id<>?2 AND ((date<?3 AND mileage>?4) OR (date>?3 AND mileage<?4) OR (date=?3 AND mileage=?4))",
                params![x.vehicle_id, x.id, x.date, x.mileage],
                |r| r.get(0),
            )
            .map_err(dberr)?;
        if bad > 0 {
            return Err(
                "The mileage is inconsistent with the other refuelings of this vehicle".to_string(),
            );
        }
        if x.id != 0 {
            self.conn
                .execute(
                    "UPDATE fuel SET vehicle_id=?1,driver_id=?2,date=?3,liters=?4,price=?5,mileage=?6,station=?7,full_tank=?8 WHERE id=?9",
                    params![
                        x.vehicle_id, nz(x.driver_id), x.date, x.liters, x.price, x.mileage,
                        x.station, x.full_tank as i64, x.id
                    ],
                )
                .map_err(dberr)?;
        } else {
            self.conn
                .execute(
                    "INSERT INTO fuel(vehicle_id,driver_id,date,liters,price,mileage,station,full_tank) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                    params![
                        x.vehicle_id, nz(x.driver_id), x.date, x.liters, x.price, x.mileage,
                        x.station, x.full_tank as i64
                    ],
                )
                .map_err(dberr)?;
        }
        let id = if x.id != 0 {
            x.id
        } else {
            self.conn.last_insert_rowid()
        };
        self.bump_mileage(x.vehicle_id, x.mileage)?;
        Ok(id)
    }

    pub fn delete_fuel(&self, id: i64) -> R<()> {
        self.conn
            .execute("DELETE FROM fuel WHERE id=?1", params![id])
            .map_err(dberr)?;
        Ok(())
    }

    pub fn list_expenses(&self) -> R<Vec<Expense>> {
        query_list!(
            self,
            "SELECT id,vehicle_id,category,date,amount,description FROM expenses ORDER BY date DESC,id DESC",
            |r| Ok(Expense {
                id: r.get(0)?,
                vehicle_id: r.get(1)?,
                category: r.get(2)?,
                date: r.get(3)?,
                amount: r.get(4)?,
                description: r.get(5)?,
            })
        )
    }

    pub fn save_expense(&self, x: &Expense) -> R<i64> {
        if x.vehicle_id == 0 {
            return Err("Select a vehicle".to_string());
        }
        if x.amount <= 0.0 {
            return Err("The amount must be greater than zero".to_string());
        }
        if x.id != 0 {
            self.conn
                .execute(
                    "UPDATE expenses SET vehicle_id=?1,category=?2,date=?3,amount=?4,description=?5 WHERE id=?6",
                    params![x.vehicle_id, x.category, x.date, x.amount, x.description, x.id],
                )
                .map_err(dberr)?;
            Ok(x.id)
        } else {
            self.conn
                .execute(
                    "INSERT INTO expenses(vehicle_id,category,date,amount,description) VALUES(?1,?2,?3,?4,?5)",
                    params![x.vehicle_id, x.category, x.date, x.amount, x.description],
                )
                .map_err(dberr)?;
            Ok(self.conn.last_insert_rowid())
        }
    }

    pub fn delete_expense(&self, id: i64) -> R<()> {
        self.conn
            .execute("DELETE FROM expenses WHERE id=?1", params![id])
            .map_err(dberr)?;
        Ok(())
    }
}
