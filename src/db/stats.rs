use super::cache::Cache;
use super::models::Fuel;
use crate::util::format as f;
use chrono::Datelike;

#[derive(Default, Clone)]
pub struct Consumption {
    pub l100: f64,
    pub cost_km: f64,
    pub all_km: f64,
    pub km: i64,
    pub liters: f64,
    pub cost: f64,
}

#[derive(Default)]
pub struct Stats {
    pub vehicles: usize,
    pub drivers: usize,
    pub active_drivers: usize,
    pub available: usize,
    pub assigned: usize,
    pub maintenance: usize,
    pub retired: usize,
    pub expired: usize,
    pub expiring: usize,
    pub month_total: f64,
    pub month_fuel: f64,
    pub month_maint: f64,
    pub month_other: f64,
    pub month_liters: f64,
    pub avg_price: f64,
    pub fleet_l100: f64,
    pub fleet_cost_km: f64,
    pub monthly: Vec<(String, f64, f64, f64)>,
    pub by_vehicle: Vec<(String, f64)>,
}

pub fn consumption(c: &Cache, vid: i64, from: &str, to: &str) -> Option<Consumption> {
    let mut recs: Vec<&Fuel> = c
        .fuel
        .iter()
        .filter(|x| x.vehicle_id == vid && x.date.as_str() >= from && x.date.as_str() <= to)
        .collect();
    recs.sort_by(|a, b| (a.date.as_str(), a.mileage).cmp(&(b.date.as_str(), b.mileage)));
    let full: Vec<usize> = recs
        .iter()
        .enumerate()
        .filter(|(_, r)| r.full_tank)
        .map(|(i, _)| i)
        .collect();
    if full.len() < 2 {
        return None;
    }
    let fi = full[0];
    let li = *full.last()?;
    let km = recs[li].mileage - recs[fi].mileage;
    if km <= 0 {
        return None;
    }
    let liters: f64 = recs[fi + 1..=li].iter().map(|r| r.liters).sum();
    let cost: f64 = recs[fi + 1..=li].iter().map(|r| r.liters * r.price).sum();
    let d0 = recs[fi].date.clone();
    let d1 = recs[li].date.clone();
    let exp: f64 = c
        .expenses
        .iter()
        .filter(|e| e.vehicle_id == vid && e.date > d0 && e.date <= d1)
        .map(|e| e.amount)
        .sum();
    let mnt: f64 = c
        .maintenance
        .iter()
        .filter(|m| {
            m.vehicle_id == vid
                && m.status == "Completed"
                && m.completed_date > d0
                && m.completed_date <= d1
        })
        .map(|m| m.cost)
        .sum();
    let kmf = km as f64;
    Some(Consumption {
        l100: liters / kmf * 100.0,
        cost_km: cost / kmf,
        all_km: (cost + exp + mnt) / kmf,
        km,
        liters,
        cost,
    })
}

pub fn compute(c: &Cache) -> Stats {
    let mut s = Stats {
        vehicles: c.vehicles.len(),
        drivers: c.drivers.len(),
        active_drivers: c.drivers.iter().filter(|d| d.active).count(),
        ..Default::default()
    };
    for v in &c.vehicles {
        match v.status.as_str() {
            "Available" => s.available += 1,
            "Assigned" => s.assigned += 1,
            "Maintenance" => s.maintenance += 1,
            _ => s.retired += 1,
        }
    }
    s.expired = c.deadlines.iter().filter(|d| d.level == 2).count();
    s.expiring = c.deadlines.iter().filter(|d| d.level == 1).count();

    let today = f::today();
    let key = today.format("%Y-%m").to_string();
    for x in c.fuel.iter().filter(|x| x.date.starts_with(&key)) {
        s.month_fuel += x.liters * x.price;
        s.month_liters += x.liters;
    }
    for m in c
        .maintenance
        .iter()
        .filter(|m| m.status == "Completed" && m.completed_date.starts_with(&key))
    {
        s.month_maint += m.cost;
    }
    for e in c.expenses.iter().filter(|e| e.date.starts_with(&key)) {
        s.month_other += e.amount;
    }
    s.month_total = s.month_fuel + s.month_maint + s.month_other;
    if s.month_liters > 0.0 {
        s.avg_price = s.month_fuel / s.month_liters;
    }

    let mut y = today.year();
    let mut m = today.month();
    let mut keys: Vec<(String, String)> = Vec::new();
    for _ in 0..6 {
        let names = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        keys.push((
            format!("{:04}-{:02}", y, m),
            names[(m - 1) as usize].to_string(),
        ));
        if m == 1 {
            m = 12;
            y -= 1;
        } else {
            m -= 1;
        }
    }
    keys.reverse();
    for (k, label) in keys {
        let fuel: f64 = c
            .fuel
            .iter()
            .filter(|x| x.date.starts_with(&k))
            .map(|x| x.liters * x.price)
            .sum();
        let maint: f64 = c
            .maintenance
            .iter()
            .filter(|x| x.status == "Completed" && x.completed_date.starts_with(&k))
            .map(|x| x.cost)
            .sum();
        let other: f64 = c
            .expenses
            .iter()
            .filter(|x| x.date.starts_with(&k))
            .map(|x| x.amount)
            .sum();
        s.monthly.push((label, fuel, maint, other));
    }

    let (mut tl, mut tk, mut tc) = (0.0, 0.0, 0.0);
    for v in &c.vehicles {
        if let Some(cs) = consumption(c, v.id, "0000-01-01", "9999-12-31") {
            s.by_vehicle.push((v.plate.clone(), cs.l100));
            tl += cs.liters;
            tk += cs.km as f64;
            tc += cs.cost;
        }
    }
    if tk > 0.0 {
        s.fleet_l100 = tl / tk * 100.0;
        s.fleet_cost_km = tc / tk;
    }
    s
}
