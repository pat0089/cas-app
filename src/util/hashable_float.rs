use std::f64::EPSILON;
use std::hash::{Hash, Hasher};

#[derive(Debug, Copy, Clone)]
pub struct HashableFloat(pub f64);

impl HashableFloat {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    // Round to nearest multiple of epsilon for comparison
    fn normalized(&self) -> f64 {
        (self.0 / EPSILON).round() * EPSILON
    }

    // Relative epsilon comparison
    fn approx_eq(&self, other: &Self) -> bool {
        if self.0 == other.0 {
            return true;
        } // Exact equality including infinities
        if self.0.is_nan() && other.0.is_nan() {
            return true;
        }

        let abs_a = self.0.abs();
        let abs_b = other.0.abs();
        let diff = (self.0 - other.0).abs();

        if abs_a == 0.0 || abs_b == 0.0 {
            // Handle cases near zero
            diff < EPSILON
        } else {
            // Relative epsilon comparison
            diff / abs_a.max(abs_b) < EPSILON
        }
    }
}

impl PartialEq for HashableFloat {
    fn eq(&self, other: &Self) -> bool {
        self.approx_eq(other)
    }
}

impl Eq for HashableFloat {}

impl Hash for HashableFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let normalized = self.normalized();
        if normalized == 0.0 {
            // Handle negative and positive zero
            0.0f64.to_bits().hash(state);
        } else if normalized.is_nan() {
            // All NaN values hash the same
            f64::NAN.to_bits().hash(state);
        } else {
            // Normal values hash to their normalized form
            normalized.to_bits().hash(state);
        }
    }
}

impl std::fmt::Display for HashableFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_equality() {
        let a = HashableFloat::new(1.0);
        let b = HashableFloat::new(1.0 + EPSILON / 2.0);
        let c = HashableFloat::new(1.0 + EPSILON * 2.0);
        let d = HashableFloat::new(1.0 - EPSILON / 2.0);
        let e = HashableFloat::new(1.0 - EPSILON * 2.0);

        let z = HashableFloat::new(1.0 + EPSILON * 2.0 * 0.5);

        let f = HashableFloat::new(0.0);
        let g = HashableFloat::new(-0.0);

        let h = HashableFloat::new(f64::NAN);

        assert_eq!(a, b); // Should be equal within epsilon
        assert_eq!(a, d); // Should not be equal beyond epsilon
        assert_ne!(a, e); // Should not be equal beyond epsilon
        assert_ne!(a, c); // Should not be equal beyond epsilon
        assert_eq!(a, z); // Should be equal within epsilon

        assert_eq!(f, g); // Should be equal

        assert_eq!(h, h); // Should be equal
    }

    #[test]
    fn test_hash_consistency() {
        let mut map = HashMap::new();

        // Insert with one value
        map.insert(HashableFloat::new(1.0), "first");

        // Should find value with approximately equal key
        assert_eq!(
            map.get(&HashableFloat::new(1.0 + EPSILON / 2.0)),
            Some(&"first")
        );

        // Should not find value with key different beyond epsilon
        assert_eq!(map.get(&HashableFloat::new(1.0 + EPSILON * 2.0)), None);
    }

    #[test]
    fn test_special_cases() {
        let mut map = HashMap::new();

        // Test zeros
        map.insert(HashableFloat::new(0.0), "zero");
        assert_eq!(map.get(&HashableFloat::new(-0.0)), Some(&"zero"));

        // Test NaN
        map.insert(HashableFloat::new(f64::NAN), "nan");
        assert_eq!(map.get(&HashableFloat::new(f64::NAN)), Some(&"nan"));

        // Test infinities
        map.insert(HashableFloat::new(f64::INFINITY), "inf");
        assert_eq!(map.get(&HashableFloat::new(f64::INFINITY)), Some(&"inf"));
        assert_ne!(
            map.get(&HashableFloat::new(f64::NEG_INFINITY)),
            Some(&"inf")
        );
    }

    #[test]
    fn test_root_values() {
        let sqrt_2 = HashableFloat::new(2.0f64.sqrt());
        let pow_2 = HashableFloat::new(2.0f64.powf(0.5));

        assert_eq!(sqrt_2, pow_2);

        let mut map = HashMap::new();
        map.insert(sqrt_2, "sqrt2");
        assert_eq!(map.get(&pow_2), Some(&"sqrt2"));
    }
}
