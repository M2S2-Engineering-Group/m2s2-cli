pub fn to_pascal_case(input: &str) -> String {
    input
        .split(['-', '_', ' '])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None    => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

pub fn to_kebab_case(pascal: &str) -> String {
    let mut out = String::new();
    for (i, c) in pascal.chars().enumerate() {
        if c.is_uppercase() && i != 0 {
            out.push('-');
        }
        out.extend(c.to_lowercase());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pascal_from_kebab() {
        assert_eq!(to_pascal_case("hero-section"), "HeroSection");
    }

    #[test]
    fn pascal_from_snake() {
        assert_eq!(to_pascal_case("my_card"), "MyCard");
    }

    #[test]
    fn pascal_from_already_pascal() {
        assert_eq!(to_pascal_case("HeroSection"), "HeroSection");
    }

    #[test]
    fn pascal_from_single_word() {
        assert_eq!(to_pascal_case("navbar"), "Navbar");
    }

    #[test]
    fn kebab_from_pascal() {
        assert_eq!(to_kebab_case("HeroSection"), "hero-section");
    }

    #[test]
    fn kebab_from_single_word() {
        assert_eq!(to_kebab_case("Navbar"), "navbar");
    }

    #[test]
    fn kebab_from_multi_word() {
        assert_eq!(to_kebab_case("DataTableHeader"), "data-table-header");
    }

    #[test]
    fn roundtrip_kebab_to_pascal_to_kebab() {
        let input = "feature-card";
        assert_eq!(to_kebab_case(&to_pascal_case(input)), input);
    }
}
