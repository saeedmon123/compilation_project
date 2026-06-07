use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Memory {
    variables: HashMap<String, i32>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Result<i32, String> {
        self.variables
            .get(name)
            .copied()
            .ok_or_else(|| format!("Runtime error: variable '{}' is undefined", name))
    }

    pub fn set(&mut self, name: String, value: i32) {
        self.variables.insert(name, value);
    }

    pub fn all(&self) -> &HashMap<String, i32> {
        &self.variables
    }
}