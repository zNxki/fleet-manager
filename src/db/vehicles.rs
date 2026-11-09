use super::models::Vehicle;
use super::{dberr, Db, R};
use rusqlite::params;

impl Db {
    pub fn list_vehicles(&self) -> R<Vec<Vehicle>> {
        query_list!(
            self,
            "SELECT id,plate,make,model,vin,year,fuel_type,mileage,acq_date,acq_price,acq_kind,base_status,image_path,notes FROM vehicles ORDER BY plate",
            |r| Ok(Vehicle {
                id: r.get(0)?,
                plate: r.get(1)?,
                make: r.get(2)?,
                model: r.get(3)?,
                vin: r.get(4)?,
                year: r.get(5)?,
                fuel_type: r.get(6)?,
                mileage: r.get(7)?,
                acq_date: r.get(8)?,
                acq_price: r.get(9)?,
                acq_kind: r.get(10)?,
                base_status: r.get(11)?,
                image_path: r.get(12)?,
                notes: r.get(13)?,
                status: String::new(),
            })
        )
    }

    pub fn save_vehicle(&self, v: &Vehicle) -> R<i64> {
        if v.id != 0 {
            let rec = self.max_recorded_km(v.id);
            if v.mileage < rec {
                return Err(format!(
                    "Mileage cannot be lower than the highest recorded value ({})",
                    rec
                ));
            }
            if v.base_status == "Retired" && self.has_open_assignment(v.id)? {
                return Err("Return the vehicle before retiring it".to_string());
            }
            self.conn
                .execute(
                    "UPDATE vehicles SET plate=?1,make=?2,model=?3,vin=?4,year=?5,fuel_type=?6,mileage=?7,acq_date=?8,acq_price=?9,acq_kind=?10,base_status=?11,image_path=?12,notes=?13 WHERE id=?14",
                    params![
                        v.plate, v.make, v.model, v.vin, v.year, v.fuel_type, v.mileage, v.acq_date,
                        v.acq_price, v.acq_kind, v.base_status, v.image_path, v.notes, v.id
                    ],
                )
                .map_err(dberr)?;
            Ok(v.id)
        } else {
            self.conn
                .execute(
                    "INSERT INTO vehicles(plate,make,model,vin,year,fuel_type,mileage,acq_date,acq_price,acq_kind,base_status,image_path,notes) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
                    params![
                        v.plate, v.make, v.model, v.vin, v.year, v.fuel_type, v.mileage, v.acq_date,
                        v.acq_price, v.acq_kind, v.base_status, v.image_path, v.notes
                    ],
                )
                .map_err(dberr)?;
            Ok(self.conn.last_insert_rowid())
        }
    }

    pub fn delete_vehicle(&self, id: i64) -> R<()> {
        if self.has_open_assignment(id)? {
            return Err(
                "The vehicle has an open assignment: register the return first".to_string(),
            );
        }
        self.conn
            .execute("DELETE FROM vehicles WHERE id=?1", params![id])
            .map_err(dberr)?;
        Ok(())
    }
}
