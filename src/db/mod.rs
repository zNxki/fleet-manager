macro_rules! query_list {
    ($db:expr, $sql:expr, $map:expr) => {{
        let mut st = $db.conn.prepare($sql).map_err($crate::db::dberr)?;
        let rows = st.query_map([], $map).map_err($crate::db::dberr)?;
        let v = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err($crate::db::dberr)?;
        Ok(v)
    }};
}

pub mod cache;
pub mod models;
pub mod reports;
pub mod stats;
pub mod users;

mod assignments;
mod documents;
mod drivers;
mod fuel;
mod maintenance;
mod vehicles;

use models::Activity;
use rusqlite::{params, Connection, OpenFlags};
use std::path::{Path, PathBuf};

pub type R<T> = Result<T, String>;

pub fn dberr(e: rusqlite::Error) -> String {
    let s = e.to_string();
    if s.contains("UNIQUE") {
        "A record with the same unique value (plate, VIN or license number) already exists"
            .to_string()
    } else if s.contains("FOREIGN KEY") {
        "The operation conflicts with related records".to_string()
    } else {
        s
    }
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS vehicles(
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 plate TEXT NOT NULL UNIQUE COLLATE NOCASE,
 make TEXT NOT NULL,
 model TEXT NOT NULL,
 vin TEXT NOT NULL UNIQUE COLLATE NOCASE,
 year INTEGER NOT NULL,
 fuel_type TEXT NOT NULL,
 mileage INTEGER NOT NULL DEFAULT 0 CHECK(mileage>=0),
 acq_date TEXT NOT NULL DEFAULT '',
 acq_price REAL NOT NULL DEFAULT 0,
 acq_kind TEXT NOT NULL DEFAULT 'Purchase',
 base_status TEXT NOT NULL DEFAULT 'Active',
 image_path TEXT NOT NULL DEFAULT '',
 notes TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS drivers(
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 first_name TEXT NOT NULL,
 last_name TEXT NOT NULL,
 tax_code TEXT NOT NULL DEFAULT '',
 birth_date TEXT NOT NULL DEFAULT '',
 phone TEXT NOT NULL DEFAULT '',
 email TEXT NOT NULL DEFAULT '',
 address TEXT NOT NULL DEFAULT '',
 license_number TEXT NOT NULL UNIQUE COLLATE NOCASE,
 license_category TEXT NOT NULL DEFAULT 'B',
 license_expiry TEXT NOT NULL,
 hire_date TEXT NOT NULL DEFAULT '',
 job_title TEXT NOT NULL DEFAULT '',
 employment_type TEXT NOT NULL DEFAULT 'Full-time',
 active INTEGER NOT NULL DEFAULT 1,
 notes TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS assignments(
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 vehicle_id INTEGER NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
 driver_id INTEGER NOT NULL REFERENCES drivers(id) ON DELETE CASCADE,
 start_date TEXT NOT NULL,
 end_date TEXT NOT NULL DEFAULT '',
 start_km INTEGER NOT NULL CHECK(start_km>=0),
 end_km INTEGER,
 notes TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS maintenance(
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 vehicle_id INTEGER NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
 kind TEXT NOT NULL,
 description TEXT NOT NULL,
 provider TEXT NOT NULL DEFAULT '',
 scheduled_date TEXT NOT NULL DEFAULT '',
 completed_date TEXT NOT NULL DEFAULT '',
 cost REAL NOT NULL DEFAULT 0 CHECK(cost>=0),
 mileage INTEGER NOT NULL DEFAULT 0,
 status TEXT NOT NULL DEFAULT 'Scheduled',
 next_due_km INTEGER NOT NULL DEFAULT 0,
 next_due_date TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS fuel(
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 vehicle_id INTEGER NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
 driver_id INTEGER REFERENCES drivers(id) ON DELETE SET NULL,
 date TEXT NOT NULL,
 liters REAL NOT NULL CHECK(liters>0),
 price REAL NOT NULL CHECK(price>0),
 mileage INTEGER NOT NULL CHECK(mileage>=0),
 station TEXT NOT NULL DEFAULT '',
 full_tank INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS expenses(
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 vehicle_id INTEGER NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
 category TEXT NOT NULL,
 date TEXT NOT NULL,
 amount REAL NOT NULL CHECK(amount>0),
 description TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS documents(
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 vehicle_id INTEGER REFERENCES vehicles(id) ON DELETE CASCADE,
 driver_id INTEGER REFERENCES drivers(id) ON DELETE CASCADE,
 doc_type TEXT NOT NULL,
 number TEXT NOT NULL DEFAULT '',
 issuer TEXT NOT NULL DEFAULT '',
 issue_date TEXT NOT NULL DEFAULT '',
 expiry_date TEXT NOT NULL DEFAULT '',
 file_path TEXT NOT NULL DEFAULT '',
 notes TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS users(
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 username TEXT NOT NULL UNIQUE COLLATE NOCASE,
 password_hash TEXT NOT NULL,
 is_admin INTEGER NOT NULL DEFAULT 0,
 active INTEGER NOT NULL DEFAULT 1,
 must_change INTEGER NOT NULL DEFAULT 1,
 created TEXT NOT NULL,
 last_login TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS activity(
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 ts TEXT NOT NULL,
 message TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_assign_vehicle ON assignments(vehicle_id);
CREATE INDEX IF NOT EXISTS idx_assign_driver ON assignments(driver_id);
CREATE INDEX IF NOT EXISTS idx_fuel_vehicle ON fuel(vehicle_id);
CREATE INDEX IF NOT EXISTS idx_maint_vehicle ON maintenance(vehicle_id);
";

pub struct Db {
    pub conn: Connection,
    pub path: PathBuf,
}

impl Db {
    pub fn open() -> R<Db> {
        let dir = crate::util::files::data_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join("fleetmanager.db");
        let conn = Connection::open(&path).map_err(dberr)?;
        let db = Db { conn, path };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> R<()> {
        self.conn
            .execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(dberr)?;
        self.conn.execute_batch(SCHEMA).map_err(dberr)
    }

    pub fn setting(&self, key: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT value FROM settings WHERE key=?1",
                params![key],
                |r| r.get(0),
            )
            .ok()
    }

    pub fn set_setting(&self, key: &str, val: &str) {
        let _ = self.conn.execute(
            "INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, val],
        );
    }

    pub fn log(&self, msg: &str) {
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = self.conn.execute(
            "INSERT INTO activity(ts,message) VALUES(?1,?2)",
            params![ts, msg],
        );
    }

    pub fn list_activity(&self, limit: i64) -> R<Vec<Activity>> {
        let sql = format!(
            "SELECT ts,message FROM activity ORDER BY id DESC LIMIT {}",
            limit
        );
        query_list!(self, &sql, |r| Ok(Activity {
            ts: r.get(0)?,
            message: r.get(1)?,
        }))
    }

    pub fn max_recorded_km(&self, vid: i64) -> i64 {
        self.conn
            .query_row(
                "SELECT MAX(m) FROM (SELECT MAX(mileage) m FROM fuel WHERE vehicle_id=?1 \
                 UNION ALL SELECT MAX(mileage) FROM maintenance WHERE vehicle_id=?1 \
                 UNION ALL SELECT MAX(COALESCE(end_km,start_km)) FROM assignments WHERE vehicle_id=?1)",
                params![vid],
                |r| r.get::<_, Option<i64>>(0),
            )
            .ok()
            .flatten()
            .unwrap_or(0)
    }

    pub fn bump_mileage(&self, vid: i64, km: i64) -> R<()> {
        self.conn
            .execute(
                "UPDATE vehicles SET mileage=MAX(mileage,?1) WHERE id=?2",
                params![km, vid],
            )
            .map_err(dberr)?;
        Ok(())
    }

    pub fn backup(&self, dest: &Path) -> R<()> {
        if dest.exists() {
            std::fs::remove_file(dest).map_err(|e| e.to_string())?;
        }
        self.conn
            .execute(
                "VACUUM INTO ?1",
                params![dest.to_string_lossy().to_string()],
            )
            .map_err(dberr)?;
        Ok(())
    }

    pub fn validate_backup(src: &Path) -> R<()> {
        let bad = || "Not a valid FleetManager backup".to_string();
        let c = Connection::open_with_flags(src, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|_| bad())?;
        let n: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN \
                 ('vehicles','drivers','assignments','maintenance','fuel','expenses','documents')",
                [],
                |r| r.get(0),
            )
            .map_err(|_| bad())?;
        if n < 7 {
            return Err(bad());
        }
        let ok: String = c
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .map_err(|_| bad())?;
        if ok != "ok" {
            return Err("The backup file is corrupted".to_string());
        }
        Ok(())
    }

    pub fn restore(&mut self, src: &Path) -> R<()> {
        Self::validate_backup(src)?;
        let tmp = Connection::open_in_memory().map_err(dberr)?;
        let old = std::mem::replace(&mut self.conn, tmp);
        drop(old);
        let copied = std::fs::copy(src, &self.path).map_err(|e| e.to_string());
        let conn = Connection::open(&self.path).map_err(dberr)?;
        self.conn = conn;
        copied?;
        self.migrate()
    }
}
