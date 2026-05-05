pub enum SizeUnit {
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
}

impl std::str::FromStr for SizeUnit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "b" | "bytes" => Ok(Self::Bytes),
            "kb" | "kilobytes" => Ok(Self::Kilobytes),
            "mb" | "megabytes" => Ok(Self::Megabytes),
            "gb" | "gigabytes" => Ok(Self::Gigabytes),
            _ => anyhow::bail!("Invalid size unit: {}. Allowed values: b, kb, mb, gb", s),
        }
    }
}

impl SizeUnit {
    pub fn convert_bytes(&self, bytes: u64) -> f64 {
        match self {
            Self::Bytes => bytes as f64,
            Self::Kilobytes => bytes as f64 / 1024.0,
            Self::Megabytes => bytes as f64 / (1024.0 * 1024.0),
            Self::Gigabytes => bytes as f64 / (1024.0 * 1024.0 * 1024.0),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Bytes => "bytes",
            Self::Kilobytes => "kb",
            Self::Megabytes => "mb",
            Self::Gigabytes => "gb",
        }
    }
}

pub fn calculate_percentage(part: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}

pub fn apply_quantity_limit<T, F>(items: Vec<T>, quantity: Option<usize>, key_fn: F) -> Vec<T>
where
    F: Fn(&T) -> u64,
{
    if let Some(qty) = quantity {
        let mut sorted = items;
        sorted.sort_by_key(|b| std::cmp::Reverse(key_fn(b)));
        sorted.into_iter().take(qty).collect()
    } else {
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_unit_conversion() {
        let bytes = 1024 * 1024; // 1 MB

        assert_eq!(SizeUnit::Bytes.convert_bytes(bytes), 1048576.0);
        assert_eq!(SizeUnit::Kilobytes.convert_bytes(bytes), 1024.0);
        assert_eq!(SizeUnit::Megabytes.convert_bytes(bytes), 1.0);
        assert_eq!(SizeUnit::Gigabytes.convert_bytes(bytes), 1.0 / 1024.0);
    }

    #[test]
    fn test_size_unit_from_str() {
        assert!("b".parse::<SizeUnit>().is_ok());
        assert!("kb".parse::<SizeUnit>().is_ok());
        assert!("mb".parse::<SizeUnit>().is_ok());
        assert!("gb".parse::<SizeUnit>().is_ok());
        assert!("invalid".parse::<SizeUnit>().is_err());
    }

    #[test]
    fn test_calculate_percentage() {
        assert_eq!(calculate_percentage(50, 100), 50.0);
        assert_eq!(calculate_percentage(25, 100), 25.0);
        assert_eq!(calculate_percentage(0, 100), 0.0);
        assert_eq!(calculate_percentage(100, 0), 0.0); // Division by zero protection
    }

    #[test]
    fn test_apply_quantity_limit() {
        struct Item {
            value: u64,
        }

        let items = vec![
            Item { value: 100 },
            Item { value: 50 },
            Item { value: 200 },
            Item { value: 75 },
        ];

        let limited = apply_quantity_limit(items, Some(2), |item| item.value);
        assert_eq!(limited.len(), 2);
        assert_eq!(limited[0].value, 200);
        assert_eq!(limited[1].value, 100);

        let unlimited =
            apply_quantity_limit(vec![Item { value: 10 }, Item { value: 20 }], None, |item| {
                item.value
            });
        assert_eq!(unlimited.len(), 2);
    }
}
