use super::cache::Cache;
use super::models::Assignment;
use super::stats::consumption;
use crate::i18n::tr;
use crate::util::format as f;

pub const KINDS: [&str; 5] = [
    "Fleet expenses",
    "Maintenance",
    "Fuel",
    "Vehicle utilization",
    "Drivers and assignments",
];

#[derive(Clone, Default)]
pub struct Table {
    pub title: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

fn hs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| tr(s)).collect()
}

fn overlap_days(a: &Assignment, from: &str, to: &str) -> i64 {
    let today = f::today_iso();
    let s = if a.start_date.as_str() > from {
        a.start_date.as_str()
    } else {
        from
    };
    let end = if a.end_date.is_empty() {
        today.as_str()
    } else {
        a.end_date.as_str()
    };
    let e = if end < to { end } else { to };
    match (f::parse_iso(s), f::parse_iso(e)) {
        (Some(s), Some(e)) if e >= s => (e - s).num_days() + 1,
        _ => 0,
    }
}

pub fn build(kind: usize, c: &Cache, from: &str, to: &str) -> Table {
    match kind {
        0 => expenses(c, from, to),
        1 => maintenance(c, from, to),
        2 => fuel(c, from, to),
        3 => utilization(c, from, to),
        _ => drivers(c, from, to),
    }
}

fn expenses(c: &Cache, from: &str, to: &str) -> Table {
    let mut rows = Vec::new();
    let (mut tf, mut tm, mut to_) = (0.0, 0.0, 0.0);
    for v in &c.vehicles {
        let fu: f64 = c
            .fuel
            .iter()
            .filter(|x| x.vehicle_id == v.id && x.date.as_str() >= from && x.date.as_str() <= to)
            .map(|x| x.liters * x.price)
            .sum();
        let ma: f64 = c
            .maintenance
            .iter()
            .filter(|x| {
                x.vehicle_id == v.id
                    && x.status == "Completed"
                    && x.completed_date.as_str() >= from
                    && x.completed_date.as_str() <= to
            })
            .map(|x| x.cost)
            .sum();
        let ot: f64 = c
            .expenses
            .iter()
            .filter(|x| x.vehicle_id == v.id && x.date.as_str() >= from && x.date.as_str() <= to)
            .map(|x| x.amount)
            .sum();
        tf += fu;
        tm += ma;
        to_ += ot;
        rows.push(vec![
            v.plate.clone(),
            format!("{} {}", v.make, v.model),
            f::money(fu),
            f::money(ma),
            f::money(ot),
            f::money(fu + ma + ot),
        ]);
    }
    rows.push(vec![
        tr("TOTAL"),
        String::new(),
        f::money(tf),
        f::money(tm),
        f::money(to_),
        f::money(tf + tm + to_),
    ]);
    Table {
        title: tr("Fleet expenses report"),
        headers: hs(&[
            "Plate",
            "Vehicle",
            "Fuel",
            "Maintenance",
            "Other expenses",
            "Total",
        ]),
        rows,
    }
}

fn maintenance(c: &Cache, from: &str, to: &str) -> Table {
    let mut rows = Vec::new();
    let mut total = 0.0;
    let mut list: Vec<_> = c
        .maintenance
        .iter()
        .filter(|m| {
            let d = if m.completed_date.is_empty() {
                &m.scheduled_date
            } else {
                &m.completed_date
            };
            d.as_str() >= from && d.as_str() <= to
        })
        .collect();
    list.sort_by(|a, b| {
        let da = if a.completed_date.is_empty() {
            &a.scheduled_date
        } else {
            &a.completed_date
        };
        let db = if b.completed_date.is_empty() {
            &b.scheduled_date
        } else {
            &b.completed_date
        };
        da.cmp(db)
    });
    for m in list {
        let d = if m.completed_date.is_empty() {
            &m.scheduled_date
        } else {
            &m.completed_date
        };
        if m.status == "Completed" {
            total += m.cost;
        }
        rows.push(vec![
            f::date(d),
            c.vehicle_name(m.vehicle_id),
            tr(&m.kind),
            m.description.clone(),
            m.provider.clone(),
            tr(&m.status),
            f::money(m.cost),
        ]);
    }
    rows.push(vec![
        tr("TOTAL"),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        f::money(total),
    ]);
    Table {
        title: tr("Maintenance report"),
        headers: hs(&[
            "Date",
            "Vehicle",
            "Type",
            "Description",
            "Provider",
            "Status",
            "Cost",
        ]),
        rows,
    }
}

fn fuel(c: &Cache, from: &str, to: &str) -> Table {
    let mut rows = Vec::new();
    let (mut tl, mut tc, mut tn) = (0.0, 0.0, 0usize);
    for v in &c.vehicles {
        let recs: Vec<_> = c
            .fuel
            .iter()
            .filter(|x| x.vehicle_id == v.id && x.date.as_str() >= from && x.date.as_str() <= to)
            .collect();
        if recs.is_empty() {
            continue;
        }
        let liters: f64 = recs.iter().map(|x| x.liters).sum();
        let cost: f64 = recs.iter().map(|x| x.liters * x.price).sum();
        let cs = consumption(c, v.id, from, to);
        tl += liters;
        tc += cost;
        tn += recs.len();
        rows.push(vec![
            v.plate.clone(),
            recs.len().to_string(),
            f::num(liters, 2),
            f::money(cost),
            f::money(cost / liters),
            cs.as_ref()
                .map(|x| f::num(x.l100, 2))
                .unwrap_or_else(|| "-".to_string()),
            cs.as_ref()
                .map(|x| f::money(x.cost_km))
                .unwrap_or_else(|| "-".to_string()),
        ]);
    }
    rows.push(vec![
        tr("TOTAL"),
        tn.to_string(),
        f::num(tl, 2),
        f::money(tc),
        if tl > 0.0 {
            f::money(tc / tl)
        } else {
            "-".to_string()
        },
        String::new(),
        String::new(),
    ]);
    Table {
        title: tr("Fuel report"),
        headers: hs(&[
            "Plate",
            "Refuelings",
            "Liters",
            "Cost",
            "Avg price/L",
            "L/100 km",
            "Cost/km",
        ]),
        rows,
    }
}

fn utilization(c: &Cache, from: &str, to: &str) -> Table {
    let period = match (f::parse_iso(from), f::parse_iso(to)) {
        (Some(a), Some(b)) if b >= a => (b - a).num_days() + 1,
        _ => 1,
    };
    let mut rows = Vec::new();
    for v in &c.vehicles {
        let mut days = 0;
        let mut count = 0;
        let mut km = 0;
        for a in c.assignments.iter().filter(|a| a.vehicle_id == v.id) {
            let d = overlap_days(a, from, to);
            if d > 0 {
                days += d;
                count += 1;
                if let Some(e) = a.end_km {
                    km += e - a.start_km;
                }
            }
        }
        let pct = (days as f64 / period as f64 * 100.0).min(100.0);
        rows.push(vec![
            v.plate.clone(),
            format!("{} {}", v.make, v.model),
            count.to_string(),
            days.to_string(),
            format!("{:.1} %", pct),
            f::km(km),
            tr(&v.status),
        ]);
    }
    Table {
        title: tr("Vehicle utilization report"),
        headers: hs(&[
            "Plate",
            "Vehicle",
            "Assignments",
            "Days in use",
            "Utilization",
            "Distance",
            "Status",
        ]),
        rows,
    }
}

fn drivers(c: &Cache, from: &str, to: &str) -> Table {
    let mut rows = Vec::new();
    for d in &c.drivers {
        let mut days = 0;
        let mut count = 0;
        let mut km = 0;
        for a in c.assignments.iter().filter(|a| a.driver_id == d.id) {
            let n = overlap_days(a, from, to);
            if n > 0 {
                days += n;
                count += 1;
                if let Some(e) = a.end_km {
                    km += e - a.start_km;
                }
            }
        }
        let current = c
            .open_assignment_of_driver(d.id)
            .map(|a| c.vehicle_name(a.vehicle_id))
            .unwrap_or_else(|| "-".to_string());
        rows.push(vec![
            format!("{} {}", d.last_name, d.first_name),
            d.license_number.clone(),
            count.to_string(),
            days.to_string(),
            f::km(km),
            current,
            tr(if d.active { "Active" } else { "Inactive" }),
        ]);
    }
    Table {
        title: tr("Drivers and assignments report"),
        headers: hs(&[
            "Driver",
            "License",
            "Assignments",
            "Days driving",
            "Distance",
            "Current vehicle",
            "Status",
        ]),
        rows,
    }
}
