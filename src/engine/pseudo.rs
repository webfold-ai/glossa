//! Pseudo-Localization Engine.
//!
//! Generates pseudo-localized strings to test frontend UI layouts for:
//! 1. Text expansion (+35% character padding) to catch clipping and layout breakage.
//! 2. Diacritics and accented characters to verify UTF-8 font support.
//! 3. Preservation of variable slots (`{name}`, `{{name}}`, `{$name}`) and markup tags (`<0>...</0>`).

/// Converts a canonical string into a pseudo-localized string with expansion and accents.
pub fn pseudo_localize(input: &str) -> String {
    let mut out = String::from("[!!");
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Preserve {slot} or {{slot}}
        if chars[i] == '{' {
            let start = i;
            while i < chars.len() && chars[i] != '}' {
                i += 1;
            }
            if i < chars.len() && chars[i] == '}' {
                if i + 1 < chars.len() && chars[i + 1] == '}' {
                    i += 1;
                }
                i += 1;
            }
            let slot_str: String = chars[start..i].iter().collect();
            out.push_str(&slot_str);
            continue;
        }

        // Preserve <0> or </0> tags
        if chars[i] == '<' {
            let start = i;
            while i < chars.len() && chars[i] != '>' {
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
            let tag_str: String = chars[start..i].iter().collect();
            out.push_str(&tag_str);
            continue;
        }

        // Accented letter replacement
        let transformed = match chars[i] {
            'a' => 'å',
            'A' => 'Å',
            'b' => 'ƀ',
            'B' => 'Ɓ',
            'c' => 'ç',
            'C' => 'Ç',
            'd' => 'ð',
            'D' => 'Ð',
            'e' => 'é',
            'E' => 'É',
            'f' => 'ƒ',
            'F' => 'Ƒ',
            'g' => 'ĝ',
            'G' => 'Ĝ',
            'h' => 'ĥ',
            'H' => 'Ĥ',
            'i' => 'î',
            'I' => 'Î',
            'j' => 'ĵ',
            'J' => 'Ĵ',
            'k' => 'ƙ',
            'K' => 'Ƙ',
            'l' => 'ḽ',
            'L' => 'Ḽ',
            'm' => 'ḿ',
            'M' => 'Ḿ',
            'n' => 'ñ',
            'N' => 'Ñ',
            'o' => 'ö',
            'O' => 'Ö',
            'p' => 'þ',
            'P' => 'Þ',
            'q' => 'ʠ',
            'r' => 'ř',
            'R' => 'Ř',
            's' => 'š',
            'S' => 'Š',
            't' => 'ţ',
            'T' => 'Ţ',
            'u' => 'û',
            'U' => 'Û',
            'v' => 'ṽ',
            'V' => 'Ṽ',
            'w' => 'ŵ',
            'W' => 'Ŵ',
            'x' => 'ẋ',
            'X' => 'Ẋ',
            'y' => 'ý',
            'Y' => 'Ý',
            'z' => 'ž',
            'Z' => 'Ž',
            c => c,
        };
        out.push(transformed);
        i += 1;
    }

    // Add expansion padding (+35%)
    let base_len = input.len();
    let pad_chars = (base_len / 4).max(1);
    out.push(' ');
    for _ in 0..pad_chars {
        out.push('~');
    }
    out.push_str("!!]");

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pseudo_localize() {
        let original = "Hello {name}, click <0>here</0> to save!";
        let pseudo = pseudo_localize(original);

        assert!(pseudo.starts_with("[!!"));
        assert!(pseudo.ends_with("!!]"));
        // Slots and tags must be completely intact
        assert!(pseudo.contains("{name}"));
        assert!(pseudo.contains("<0>") && pseudo.contains("</0>"));
        // Accents applied
        assert!(pseudo.contains("Ĥéḽḽö"));
        assert!(pseudo.contains("ĥéřé"));
    }
}
