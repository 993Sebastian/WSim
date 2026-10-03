//! Turns messages of the core into display text (German number and date formats).

use wsim_core::calendar::Date;
use wsim_core::message::{Message, Param};
use wsim_core::money::Money;

use crate::texts::Texts;

impl Texts {
    /// Display text of a message. Missing texts show their key, so gaps stay visible.
    pub fn render(&self, message: &Message) -> String {
        let template = self.get(&message.key).unwrap_or(&message.key);
        let mut out = String::with_capacity(template.len());
        let mut rest = template;
        while let Some(start) = rest.find('{') {
            out.push_str(&rest[..start]);
            let after = &rest[start + 1..];
            let Some(end) = after.find('}') else {
                out.push_str(&rest[start..]);
                return out;
            };
            let name = &after[..end];
            match message.params.iter().find(|(n, _)| n == name) {
                Some((_, value)) => out.push_str(&self.param(value)),
                None => out.push_str(&rest[start..start + end + 2]),
            }
            rest = &after[end + 1..];
        }
        out.push_str(rest);
        out
    }

    fn param(&self, param: &Param) -> String {
        match param {
            Param::Text(text) => text.clone(),
            Param::Integer(n) => n.to_string(),
            Param::Number(n) => format_number(*n, 2),
            Param::Money(m) => format_money(*m),
            Param::Date(d) => format_date(*d),
            Param::Country(key) => self.get(&format!("land.{key}")).unwrap_or(key).to_owned(),
            Param::TextKey(key) => self.get(key).unwrap_or(key).to_owned(),
        }
    }
}

/// `1.234.567,89` – German grouping, at most `decimals` decimals, trailing zeros removed.
pub fn format_number(value: f64, decimals: usize) -> String {
    let text = format!("{:.*}", decimals, value.abs());
    let (int, frac) = text.split_once('.').unwrap_or((&text, ""));
    let mut grouped = String::new();
    for (i, digit) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            grouped.push('.');
        }
        grouped.push(digit);
    }
    let frac = frac.trim_end_matches('0');
    let sign = if value < 0.0 && (int != "0" || !frac.is_empty()) {
        "-"
    } else {
        ""
    };
    if frac.is_empty() {
        format!("{sign}{grouped}")
    } else {
        format!("{sign}{grouped},{frac}")
    }
}

/// Whole dollars from 1 000 on, cents below.
pub fn format_money(money: Money) -> String {
    let usd = money.to_usd();
    let decimals = if usd.abs() >= 1000.0 { 0 } else { 2 };
    format!("{} USD", format_number(usd, decimals))
}

pub fn format_date(date: Date) -> String {
    format!("{:02}.{:02}.{}", date.day(), date.month(), date.year())
}

#[cfg(test)]
mod tests {
    use wsim_core::message::MessageKind;

    use super::*;

    #[test]
    fn formats_numbers_the_german_way() {
        assert_eq!(format_number(1_234_567.891, 2), "1.234.567,89");
        assert_eq!(format_number(1000.0, 2), "1.000");
        assert_eq!(format_number(-12.5, 2), "-12,5");
        assert_eq!(format_number(0.0, 2), "0");
        assert_eq!(format_number(999.999, 2), "1.000");
        assert_eq!(
            format_money(Money::from_usd(250_000.0).unwrap()),
            "250.000 USD"
        );
        assert_eq!(format_money(Money::from_usd(12.3).unwrap()), "12,3 USD");
        assert_eq!(format_date(Date::new(1900, 4, 1).unwrap()), "01.04.1900");
    }

    #[test]
    fn renders_placeholders() {
        let texts = Texts::from_pairs([
            ("a.b", "{name} kauft in {land} für {betrag} ({fehlt})"),
            ("land.DEU", "Deutschland"),
        ]);
        let message = Message::new(MessageKind::Info, "a.b")
            .with("name", Param::Text("Krupp".into()))
            .with("land", Param::Country("DEU".into()))
            .with(
                "betrag",
                Param::Money(Money::from_usd(1_500_000.0).unwrap()),
            );
        assert_eq!(
            texts.render(&message),
            "Krupp kauft in Deutschland für 1.500.000 USD ({fehlt})"
        );
        assert_eq!(
            texts.render(&Message::new(MessageKind::Info, "gibt.es.nicht")),
            "gibt.es.nicht"
        );
    }
}
