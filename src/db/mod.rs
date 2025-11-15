use std::collections::HashMap;

pub struct Db {
    store: HashMap<String, String>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.store.get(key).cloned()
    }

    pub fn set(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }

    pub fn del(&mut self, key: &str) -> bool {
        if let Some(_) = self.store.remove(key) {
            return true;
        } else {
            return false;
        }
    }
}
