use crate::parser::Command;
use crate::db::Db;

pub struct Executor<'a> {
    db: &'a mut Db,
}

#[derive(Debug)]
pub enum ExecError {
    KeyNotFound,
    InvalidValue,
}

impl<'a> Executor<'a> {
    pub fn new(db: &'a mut Db) -> Self {
        Self { db }
    }

    pub fn execute(&mut self, cmd: Command) -> Result<String, ExecError> {
        match cmd {
            Command::Ping => Ok("PONG".into()),

            Command::Get(key) => {
                self.db.get(&key).ok_or(ExecError::KeyNotFound)
            }

            Command::Set(key, value) => {
                self.db.set(key, value);
                Ok("OK".into())
            }

            Command::Del(key) => {
                if self.db.del(&key) {
                    Ok("1".into())
                } else {
                    Ok("0".into())
                }
            }

            Command::Exists(key) => {
                Ok((self.db.get(&key).is_some() as i32).to_string())
            }

            Command::Inc(key) => {
                let val = self.db.get(&key).unwrap_or("0".into());
                let n: i64 = val.parse().map_err(|_| ExecError::InvalidValue)?;
                let new = n + 1;
                self.db.set(key, new.to_string());
                Ok(new.to_string())
            }
        }
    }
}