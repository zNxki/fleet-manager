use super::{dberr, Db, R};
use crate::util::files::data_dir;
use argon2::password_hash::rand_core::{OsRng, RngCore};
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rusqlite::{params, OptionalExtension, Row};
use std::path::PathBuf;

pub const MIN_PASSWORD_LEN: usize = 8;
const TEMP_PASSWORD_LEN: usize = 12;
// No look-alike characters (I, O, l, 0, 1) so temporary passwords are easy to read out.
const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789";
const USER_COLS: &str = "id,username,is_admin,active,must_change,created,last_login";

#[derive(Clone, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
    pub active: bool,
    pub must_change: bool,
    pub created: String,
    pub last_login: String,
}

fn map_user(r: &Row) -> rusqlite::Result<User> {
    Ok(User {
        id: r.get(0)?,
        username: r.get(1)?,
        is_admin: r.get::<_, i64>(2)? != 0,
        active: r.get::<_, i64>(3)? != 0,
        must_change: r.get::<_, i64>(4)? != 0,
        created: r.get(5)?,
        last_login: r.get(6)?,
    })
}

fn hash_password(pw: &str) -> R<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pw.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

fn verify_password(pw: &str, stored: &str) -> bool {
    PasswordHash::new(stored)
        .map(|h| Argon2::default().verify_password(pw.as_bytes(), &h).is_ok())
        .unwrap_or(false)
}

/// Random temporary password from the OS CSPRNG (rejection sampling, no modulo bias).
pub fn random_password() -> String {
    let limit = 256 - (256 % ALPHABET.len());
    let mut out = String::with_capacity(TEMP_PASSWORD_LEN);
    let mut b = [0u8; 1];
    while out.len() < TEMP_PASSWORD_LEN {
        OsRng.fill_bytes(&mut b);
        let v = b[0] as usize;
        if v < limit {
            out.push(ALPHABET[v % ALPHABET.len()] as char);
        }
    }
    out
}

fn validate_username(s: &str) -> R<String> {
    let name = s.trim().to_string();
    let n = name.chars().count();
    let ok = (3..=32).contains(&n)
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if ok {
        Ok(name)
    } else {
        Err("Username must be 3-32 characters (letters, numbers, . _ -)".to_string())
    }
}

pub fn initial_password_path() -> PathBuf {
    data_dir().join("admin_initial_password.txt")
}

fn write_initial_file(pw: &str) {
    let text = format!(
        "FleetManager - temporary administrator credentials\n\
         username: admin\n\
         password: {pw}\n\n\
         You must change this password at first sign-in.\n\
         This file is deleted automatically once the password has been changed.\n"
    );
    let _ = std::fs::create_dir_all(data_dir());
    let path = initial_password_path();
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)
        {
            let _ = f.write_all(text.as_bytes());
        }
    }
    #[cfg(not(unix))]
    {
        let _ = std::fs::write(&path, text);
    }
}

impl Db {
    pub fn user_count(&self) -> R<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))
            .map_err(dberr)
    }

    fn insert_user(&self, username: &str, pw: &str, admin: bool) -> R<()> {
        let hash = hash_password(pw)?;
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.conn
            .execute(
                "INSERT INTO users(username,password_hash,is_admin,active,must_change,created) \
                 VALUES(?1,?2,?3,1,1,?4)",
                params![username, hash, admin as i64, ts],
            )
            .map_err(dberr)?;
        Ok(())
    }

    /// On first run (no users at all) creates the `admin` account with a random
    /// temporary password and returns it. Returns `None` when users already exist.
    pub fn bootstrap_admin(&self) -> R<Option<String>> {
        if self.user_count()? > 0 {
            return Ok(None);
        }
        let pw = random_password();
        self.insert_user("admin", &pw, true)?;
        write_initial_file(&pw);
        self.log("Administrator account created");
        Ok(Some(pw))
    }

    /// Temporary admin password, only while the admin has not changed it yet.
    pub fn pending_initial_password(&self) -> Option<String> {
        let hash: String = self
            .conn
            .query_row(
                "SELECT password_hash FROM users WHERE username='admin' AND must_change=1",
                [],
                |r| r.get(0),
            )
            .ok()?;
        let text = std::fs::read_to_string(initial_password_path()).ok()?;
        let pw = text
            .lines()
            .find_map(|l| l.strip_prefix("password: "))?
            .trim()
            .to_string();
        verify_password(&pw, &hash).then_some(pw)
    }

    pub fn authenticate(&self, username: &str, password: &str) -> R<User> {
        let invalid = || "Invalid username or password".to_string();
        let sql = format!("SELECT {USER_COLS},password_hash FROM users WHERE username=?1");
        let row = self
            .conn
            .query_row(&sql, params![username.trim()], |r| {
                Ok((map_user(r)?, r.get::<_, String>(7)?))
            })
            .optional()
            .map_err(dberr)?;
        match row {
            None => {
                // spend comparable time so response time does not reveal valid usernames
                let _ = hash_password(password);
                Err(invalid())
            }
            Some((mut u, h)) => {
                if !verify_password(password, &h) {
                    return Err(invalid());
                }
                if !u.active {
                    return Err("This account is disabled".to_string());
                }
                let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = self.conn.execute(
                    "UPDATE users SET last_login=?1 WHERE id=?2",
                    params![ts, u.id],
                );
                u.last_login = ts;
                self.log(&format!("User signed in: {}", u.username));
                Ok(u)
            }
        }
    }

    pub fn get_user(&self, id: i64) -> R<Option<User>> {
        let sql = format!("SELECT {USER_COLS} FROM users WHERE id=?1");
        self.conn
            .query_row(&sql, params![id], map_user)
            .optional()
            .map_err(dberr)
    }

    pub fn list_users(&self) -> R<Vec<User>> {
        let sql = format!("SELECT {USER_COLS} FROM users ORDER BY username COLLATE NOCASE");
        query_list!(self, &sql, map_user)
    }

    fn stored_hash(&self, id: i64) -> R<String> {
        self.conn
            .query_row(
                "SELECT password_hash FROM users WHERE id=?1",
                params![id],
                |r| r.get(0),
            )
            .map_err(|_| "User not found".to_string())
    }

    /// Changes a password. `current` must be given for a voluntary change; the forced
    /// first-login change passes `None` because the user has just authenticated.
    pub fn change_password(
        &self,
        id: i64,
        current: Option<&str>,
        new: &str,
        confirm: &str,
    ) -> R<()> {
        let user = self.get_user(id)?.ok_or("User not found")?;
        let stored = self.stored_hash(id)?;
        if let Some(cur) = current {
            if !verify_password(cur, &stored) {
                return Err("Current password is incorrect".to_string());
            }
        }
        if new != confirm {
            return Err("Passwords do not match".to_string());
        }
        if new.chars().count() < MIN_PASSWORD_LEN {
            return Err("The password must be at least 8 characters long".to_string());
        }
        if new.eq_ignore_ascii_case(&user.username) {
            return Err("The password cannot be the same as the username".to_string());
        }
        if verify_password(new, &stored) {
            return Err("The new password must be different from the current one".to_string());
        }
        let hash = hash_password(new)?;
        self.conn
            .execute(
                "UPDATE users SET password_hash=?1, must_change=0 WHERE id=?2",
                params![hash, id],
            )
            .map_err(dberr)?;
        if user.is_admin {
            let _ = std::fs::remove_file(initial_password_path());
        }
        self.log(&format!("Password changed: {}", user.username));
        Ok(())
    }

    /// Creates a user with a random temporary password (returned once, never stored).
    pub fn create_user(&self, username: &str, admin: bool) -> R<String> {
        let name = validate_username(username)?;
        let exists: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM users WHERE username=?1",
                params![name],
                |r| r.get(0),
            )
            .map_err(dberr)?;
        if exists > 0 {
            return Err("A user with this username already exists".to_string());
        }
        let pw = random_password();
        self.insert_user(&name, &pw, admin)?;
        self.log(&format!("User created: {name}"));
        Ok(pw)
    }

    pub fn reset_password(&self, id: i64) -> R<String> {
        let user = self.get_user(id)?.ok_or("User not found")?;
        let pw = random_password();
        let hash = hash_password(&pw)?;
        self.conn
            .execute(
                "UPDATE users SET password_hash=?1, must_change=1 WHERE id=?2",
                params![hash, id],
            )
            .map_err(dberr)?;
        self.log(&format!("Password reset: {}", user.username));
        Ok(pw)
    }

    fn ensure_other_admin(&self, id: i64) -> R<()> {
        let user = self.get_user(id)?.ok_or("User not found")?;
        if user.is_admin && user.active {
            let others: i64 = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM users WHERE is_admin=1 AND active=1 AND id<>?1",
                    params![id],
                    |r| r.get(0),
                )
                .map_err(dberr)?;
            if others == 0 {
                return Err("At least one active administrator is required".to_string());
            }
        }
        Ok(())
    }

    pub fn set_user_active(&self, id: i64, active: bool) -> R<()> {
        if !active {
            self.ensure_other_admin(id)?;
        }
        self.conn
            .execute(
                "UPDATE users SET active=?1 WHERE id=?2",
                params![active as i64, id],
            )
            .map_err(dberr)?;
        Ok(())
    }

    pub fn delete_user(&self, id: i64) -> R<()> {
        self.ensure_other_admin(id)?;
        self.conn
            .execute("DELETE FROM users WHERE id=?1", params![id])
            .map_err(dberr)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn db() -> Db {
        let dir = std::env::temp_dir().join(format!("fm_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("XDG_DATA_HOME", &dir);
        std::env::remove_var("APPDATA");
        let d = Db {
            conn: Connection::open_in_memory().unwrap(),
            path: dir.join("t.db"),
        };
        d.migrate().unwrap();
        d
    }

    #[test]
    fn full_flow() {
        let d = db();
        // first start: admin with random temporary password
        let tmp = d.bootstrap_admin().unwrap().expect("admin created");
        assert_eq!(tmp.len(), 12);
        assert!(d.bootstrap_admin().unwrap().is_none());
        assert_eq!(d.pending_initial_password().as_deref(), Some(tmp.as_str()));
        assert_eq!(d.user_count().unwrap(), 1);

        // wrong password / unknown user give the same error
        assert_eq!(d.authenticate("admin", "nope").unwrap_err(), "Invalid username or password");
        assert_eq!(d.authenticate("ghost", "nope").unwrap_err(), "Invalid username or password");
        // username is case-insensitive
        let u = d.authenticate("ADMIN", &tmp).unwrap();
        assert!(u.is_admin && u.must_change);

        // forced change rules
        assert_eq!(d.change_password(u.id, None, "short", "short").unwrap_err(), "The password must be at least 8 characters long");
        assert_eq!(d.change_password(u.id, None, "longenough1", "different1").unwrap_err(), "Passwords do not match");
        assert_eq!(d.change_password(u.id, None, &tmp, &tmp).unwrap_err(), "The new password must be different from the current one");
        d.change_password(u.id, None, "My-new-pass1", "My-new-pass1").unwrap();
        assert!(!initial_password_path().exists());
        assert!(d.pending_initial_password().is_none());
        assert!(d.authenticate("admin", &tmp).is_err());
        let u = d.authenticate("admin", "My-new-pass1").unwrap();
        assert!(!u.must_change);

        // voluntary change needs the current password
        assert_eq!(d.change_password(u.id, Some("bad"), "Another-pass2", "Another-pass2").unwrap_err(), "Current password is incorrect");

        // admin creates a user: temp password, must change, usernames unique
        let pw = d.create_user("  mario.rossi ", false).unwrap();
        assert!(d.create_user("MARIO.ROSSI", false).is_err());
        assert!(d.create_user("a b", false).is_err());
        let m = d.authenticate("mario.rossi", &pw).unwrap();
        assert!(m.must_change && !m.is_admin);
        d.change_password(m.id, None, "Mario-pass-9", "Mario-pass-9").unwrap();

        // reset forces a new change; old password stops working
        let pw2 = d.reset_password(m.id).unwrap();
        assert!(d.authenticate("mario.rossi", "Mario-pass-9").is_err());
        assert!(d.authenticate("mario.rossi", &pw2).unwrap().must_change);

        // disabled users cannot sign in
        d.set_user_active(m.id, false).unwrap();
        assert_eq!(d.authenticate("mario.rossi", &pw2).unwrap_err(), "This account is disabled");
        d.set_user_active(m.id, true).unwrap();

        // the last active admin can be neither disabled nor deleted
        assert!(d.set_user_active(u.id, false).is_err());
        assert!(d.delete_user(u.id).is_err());
        d.delete_user(m.id).unwrap();
        assert_eq!(d.list_users().unwrap().len(), 1);

        // passwords are stored hashed, never in clear
        let h: String = d.conn.query_row("SELECT password_hash FROM users WHERE id=?1", params![u.id], |r| r.get(0)).unwrap();
        assert!(h.starts_with("$argon2") && !h.contains("My-new-pass1"));
    }

    #[test]
    fn random_passwords_differ_and_use_alphabet() {
        let a = random_password();
        let b = random_password();
        assert_ne!(a, b);
        assert!(a.bytes().all(|c| ALPHABET.contains(&c)));
    }
}
