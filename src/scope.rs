use std::collections::HashMap;

pub struct Scope {
    depth: i64,
    smallest_depth: i64,
    storage: HashMap<i64, HashMap<String, String> >,
}

impl Scope {

    pub fn new() -> Scope {
        Scope {
            depth: 0,
            smallest_depth: 0,
            storage: HashMap::new(),
        }
    }

    pub fn increase_depth(&mut self) {
        self.depth += 1;
    }

    pub fn decrease_depth(&mut self) {
        // Depth can go below zero because a diff might contain code that
        // starts in deeply nested could and progress to less nested.
        self.depth -= 1;

        if self.depth < self.smallest_depth {
            self.smallest_depth = self.depth;
        }
    }

    pub fn reset_depth(&mut self) {
        self.depth = 0;
    }

    // TODO Needing to clone() is ugly.
    //      So maybe original should be a str slice?
    pub fn add_variable(&mut self, original: String) { //-> String {

        {
            if let Some(sub_map) = self.storage.get_mut(&self.depth) {
                let replacement = format!("s{}v{}", self.depth, sub_map.len());
                sub_map.insert(original, replacement);

                // TODO Return replacement?
                return;
            }
        }

        let mut sub_map : HashMap<String, String> = HashMap::new();
        let replacement = format!("s{}v0", self.depth);
        sub_map.insert(original, replacement);
        self.storage.insert(self.depth, sub_map);
        // TODO Return replacement?

    }

    pub fn get_variable(&self, original: String) -> Option<String> {
        let mut depth_index : i64 = self.depth;

        loop {
            if depth_index < self.smallest_depth {
                break;
            }

            if let Some(sub_map) = self.storage.get(&depth_index) {
                if let Some(replacement) = sub_map.get(&original) {
                    return Some(replacement.clone());   // TODO Clone?
                }
            }

            depth_index -= 1;
        }

        None
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_variable_non_existant() {
        let s = Scope::new();
        assert_eq!(s.get_variable(String::new()), None);
    }

    #[test]
    fn get_variable_single_depth_single_var() {
        // Start of code (to be 'parsed' by this test).
        let a = 0;
        assert_eq!(a, 0);
        // End of code

        let mut s = Scope::new();
        let original = String::from_str("original");
        let expected = String::from_str("s0v0");
        s.add_variable(original.clone());   // TODO Clone is ugly.
        assert_eq!(s.get_variable(original), Some(expected));

// TODO Test multiple calls.
    }

    #[test]
    fn get_variable_single_depth_multiple_var() {
        let mut s = Scope::new();
        let original0 = String::from_str("original0");
        let expected0 = String::from_str("s0v0");
        let original1 = String::from_str("original1");
        let expected1 = String::from_str("s0v1");
        let original2 = String::from_str("original2");
        let expected2 = String::from_str("s0v2");
        s.add_variable(original0.clone());  // TODO Clone is ugly.
        s.add_variable(original1.clone());  // TODO Clone is ugly.
        s.add_variable(original2.clone());  // TODO Clone is ugly.
        assert_eq!(s.get_variable(original0), Some(expected0));
        assert_eq!(s.get_variable(original2), Some(expected2));
        assert_eq!(s.get_variable(original1), Some(expected1)); // TODO Order doesn't matter.
    }

    #[test]
    fn get_variable_multiple_depths_multiple_var() {
        // Start of example code that is being 'parsed' by this test.
        let a = 0;
        let b = 1;
        {
            let a = 2;
            assert_eq!(a, 2);
            assert_eq!(b, 1);
        }
        let c = 3;
        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(c, 3);
        // End of example code

        let mut s = Scope::new();

        s.add_variable(format!("a"));
        s.add_variable(format!("b"));
        s.increase_depth();
        s.add_variable(format!("a"));
        s.decrease_depth();
        s.add_variable(format!("c"));

        s.reset_depth();

        assert_eq!(
            s.get_variable(format!("a")),
            Some(format!("s0v0"))
        );
        assert_eq!(
            s.get_variable(format!("b")),
            Some(format!("s0v1"))
        );
        s.increase_depth();
        assert_eq!(
            s.get_variable(format!("a")),
            Some(format!("s1v0"))
        );
        assert_eq!(
            s.get_variable(format!("b")),
            Some(format!("s0v1"))
        );
        s.decrease_depth();
        assert_eq!(
            s.get_variable(format!("a")),
            Some(format!("s0v0"))
        );
        assert_eq!(
            s.get_variable(format!("b")),
            Some(format!("s0v1"))
        );
        assert_eq!(
            s.get_variable(format!("c")),
            Some(format!("s0v2"))
        );
    }
}

