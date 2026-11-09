use super::models::*;
use super::stats::{self, Stats};
use super::{Db, R};
use crate::util::format as f;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct Deadline {
    pub kind: String,
    pub subject: String,
    pub label: String,
    pub days: i64,
    pub level: u8,
    pub detail: String,
}

#[derive(Default)]
pub struct Cache {
    pub vehicles: Vec<Vehicle>,
    pub drivers: Vec<Driver>,
    pub assignments: Vec<Assignment>,
    pub maintenance: Vec<Maintenance>,
    pub fuel: Vec<Fuel>,
    pub expenses: Vec<Expense>,
    pub documents: Vec<Document>,
    pub activities: Vec<Activity>,
    pub deadlines: Vec<Deadline>,
    pub stats: Stats,
}

fn dl(kind: &str, subject: String, label: String, date: &str) -> Option<Deadline> {
    let days = f::days_until(date)?;
    let level = if days < 0 {
        2
    } else if days <= 30 {
        1
    } else {
        0
    };
    Some(Deadline {
        kind: kind.to_string(),
        subject,
        label,
        days,
        level,
        detail: f::date(date),
    })
}

impl Cache {
    pub fn load(db: &Db) -> R<Cache> {
        let mut c = Cache {
            vehicles: db.list_vehicles()?,
            drivers: db.list_drivers()?,
            assignments: db.list_assignments()?,
            maintenance: db.list_maintenance()?,
            fuel: db.list_fuel()?,
            expenses: db.list_expenses()?,
            documents: db.list_documents()?,
            activities: db.list_activity(40)?,
            deadlines: Vec::new(),
            stats: Stats::default(),
        };
        c.finalize();
        c.stats = stats::compute(&c);
        Ok(c)
    }

    pub fn vehicle(&self, id: i64) -> Option<&Vehicle> {
        self.vehicles.iter().find(|v| v.id == id)
    }

    pub fn driver(&self, id: i64) -> Option<&Driver> {
        self.drivers.iter().find(|d| d.id == id)
    }

    pub fn vehicle_name(&self, id: i64) -> String {
        self.vehicle(id)
            .map(|v| v.plate.clone())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn driver_name(&self, id: i64) -> String {
        self.driver(id)
            .map(|d| format!("{} {}", d.last_name, d.first_name))
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn vehicle_items(&self, all: bool) -> Vec<(i64, String)> {
        self.vehicles
            .iter()
            .filter(|v| all || v.base_status != "Retired")
            .map(|v| (v.id, format!("{} - {} {}", v.plate, v.make, v.model)))
            .collect()
    }

    pub fn driver_items(&self, all: bool) -> Vec<(i64, String)> {
        self.drivers
            .iter()
            .filter(|d| all || d.active)
            .map(|d| (d.id, format!("{} {}", d.last_name, d.first_name)))
            .collect()
    }

    pub fn open_assignment_of_vehicle(&self, vid: i64) -> Option<&Assignment> {
        self.assignments
            .iter()
            .find(|a| a.vehicle_id == vid && a.end_date.is_empty())
    }

    pub fn open_assignment_of_driver(&self, did: i64) -> Option<&Assignment> {
        self.assignments
            .iter()
            .find(|a| a.driver_id == did && a.end_date.is_empty())
    }

    fn finalize(&mut self) {
        let open: HashSet<i64> = self
            .assignments
            .iter()
            .filter(|a| a.end_date.is_empty())
            .map(|a| a.vehicle_id)
            .collect();
        let in_maint: HashSet<i64> = self
            .maintenance
            .iter()
            .filter(|m| m.status == "In progress")
            .map(|m| m.vehicle_id)
            .collect();
        for v in self.vehicles.iter_mut() {
            v.status = if v.base_status == "Retired" {
                "Retired"
            } else if in_maint.contains(&v.id) {
                "Maintenance"
            } else if open.contains(&v.id) {
                "Assigned"
            } else {
                "Available"
            }
            .to_string();
        }
        let mut out: Vec<Deadline> = Vec::new();
        {
            let mut latest: HashMap<(i64, i64, String), &Document> = HashMap::new();
            for d in &self.documents {
                if d.expiry_date.is_empty() {
                    continue;
                }
                let k = (d.vehicle_id, d.driver_id, d.doc_type.clone());
                let replace = match latest.get(&k) {
                    Some(o) => o.expiry_date < d.expiry_date,
                    None => true,
                };
                if replace {
                    latest.insert(k, d);
                }
            }
            for d in latest.values() {
                let subject = if d.vehicle_id != 0 {
                    self.vehicle_name(d.vehicle_id)
                } else {
                    self.driver_name(d.driver_id)
                };
                let label = if d.number.is_empty() {
                    d.doc_type.clone()
                } else {
                    format!("{} {}", d.doc_type, d.number)
                };
                if let Some(x) = dl("Document", subject, label, &d.expiry_date) {
                    out.push(x);
                }
            }
            for d in self.drivers.iter().filter(|d| d.active) {
                if let Some(x) = dl(
                    "License",
                    self.driver_name(d.id),
                    format!("Driving license {}", d.license_number),
                    &d.license_expiry,
                ) {
                    out.push(x);
                }
            }
            for m in self.maintenance.iter().filter(|m| m.status == "Scheduled") {
                if let Some(x) = dl(
                    "Maintenance",
                    self.vehicle_name(m.vehicle_id),
                    m.description.clone(),
                    &m.scheduled_date,
                ) {
                    out.push(x);
                }
            }
            for v in self.vehicles.iter().filter(|v| v.base_status != "Retired") {
                let last = self
                    .maintenance
                    .iter()
                    .filter(|m| m.vehicle_id == v.id && m.status == "Completed")
                    .max_by(|a, b| {
                        (a.completed_date.as_str(), a.id).cmp(&(b.completed_date.as_str(), b.id))
                    });
                if let Some(m) = last {
                    if !m.next_due_date.is_empty() {
                        if let Some(x) = dl(
                            "Maintenance",
                            v.plate.clone(),
                            format!("Next: {}", m.description),
                            &m.next_due_date,
                        ) {
                            out.push(x);
                        }
                    }
                    if m.next_due_km > 0 {
                        let left = m.next_due_km - v.mileage;
                        let level = if left < 0 {
                            2
                        } else if left <= 1000 {
                            1
                        } else {
                            0
                        };
                        out.push(Deadline {
                            kind: "Maintenance".to_string(),
                            subject: v.plate.clone(),
                            label: format!("Next: {}", m.description),
                            days: i64::MAX,
                            level,
                            detail: format!(
                                "{} ({} {})",
                                f::km(m.next_due_km),
                                f::km(left.abs()),
                                if left < 0 { "over" } else { "left" }
                            ),
                        });
                    }
                }
            }
        }
        out.sort_by(|a, b| b.level.cmp(&a.level).then(a.days.cmp(&b.days)));
        self.deadlines = out;
    }
}
