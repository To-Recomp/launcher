//! printf-style number formats of a slider's `format`: `%g` (default), `%d`, `%.Nf`.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberFormat {
    General,
    Integer,
    Fixed(usize),
}

pub fn parse(format: &str) -> Option<NumberFormat> {
    match format {
        "%g" => Some(NumberFormat::General),
        "%d" => Some(NumberFormat::Integer),
        _ => {
            let digits = format.strip_prefix("%.")?.strip_suffix('f')?;
            digits
                .parse()
                .ok()
                .filter(|n| *n <= 10)
                .map(NumberFormat::Fixed)
        }
    }
}

pub fn render(format: NumberFormat, value: f64) -> String {
    match format {
        NumberFormat::General => value.to_string(),
        NumberFormat::Integer => format!("{}", value.round() as i64),
        NumberFormat::Fixed(n) => format!("{value:.n$}"),
    }
}
