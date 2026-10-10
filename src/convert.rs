//! 进制、角度与单位换算。面板只维护输入，换算全部在这里完成。

use crate::engine::{format_value, ERR_INVALID, ERR_RANGE};

/// 输入缓冲的上限，避免超长数字把结果显示区挤满。
pub const MAX_INPUT_LEN: usize = 24;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConvertKind {
    Base,
    Angle,
    Unit,
}

impl ConvertKind {
    pub fn label(self) -> &'static str {
        match self {
            ConvertKind::Base => "进制",
            ConvertKind::Angle => "角度",
            ConvertKind::Unit => "单位",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            ConvertKind::Base => "base",
            ConvertKind::Angle => "angle",
            ConvertKind::Unit => "unit",
        }
    }
}

pub const KINDS: [ConvertKind; 3] = [ConvertKind::Base, ConvertKind::Angle, ConvertKind::Unit];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Base {
    Bin,
    Oct,
    Dec,
    Hex,
}

impl Base {
    pub fn label(self) -> &'static str {
        match self {
            Base::Bin => "BIN",
            Base::Oct => "OCT",
            Base::Dec => "DEC",
            Base::Hex => "HEX",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Base::Bin => "bin",
            Base::Oct => "oct",
            Base::Dec => "dec",
            Base::Hex => "hex",
        }
    }

    fn radix(self) -> u32 {
        match self {
            Base::Bin => 2,
            Base::Oct => 8,
            Base::Dec => 10,
            Base::Hex => 16,
        }
    }

    /// 该字符在当前进制下是否可用，供键盘决定是否点亮按键。
    pub fn accepts(self, ch: char) -> bool {
        Self::digit_value(ch).is_some_and(|digit| digit < self.radix())
    }

    fn digit_value(ch: char) -> Option<u32> {
        let lower = ch.to_ascii_lowercase();
        match lower {
            '0'..='9' => Some(lower as u32 - '0' as u32),
            'a'..='f' => Some(lower as u32 - 'a' as u32 + 10),
            _ => None,
        }
    }
}

pub const BASES: [Base; 4] = [Base::Bin, Base::Oct, Base::Dec, Base::Hex];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnitCategory {
    Length,
    Area,
    Volume,
    Mass,
    Speed,
    Time,
    Temperature,
}

impl UnitCategory {
    pub fn label(self) -> &'static str {
        match self {
            UnitCategory::Length => "长度",
            UnitCategory::Area => "面积",
            UnitCategory::Volume => "体积",
            UnitCategory::Mass => "重量",
            UnitCategory::Speed => "速度",
            UnitCategory::Time => "时间",
            UnitCategory::Temperature => "温度",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            UnitCategory::Length => "length",
            UnitCategory::Area => "area",
            UnitCategory::Volume => "volume",
            UnitCategory::Mass => "mass",
            UnitCategory::Speed => "speed",
            UnitCategory::Time => "time",
            UnitCategory::Temperature => "temperature",
        }
    }

    fn group(self) -> &'static UnitGroup {
        GROUPS
            .iter()
            .find(|group| group.category == self)
            .unwrap_or(&GROUPS[0])
    }

    pub fn units(self) -> &'static [Unit] {
        &self.group().units
    }
}

pub const CATEGORIES: [UnitCategory; 7] = [
    UnitCategory::Length,
    UnitCategory::Area,
    UnitCategory::Volume,
    UnitCategory::Mass,
    UnitCategory::Speed,
    UnitCategory::Time,
    UnitCategory::Temperature,
];

#[derive(Clone, Copy)]
pub struct Unit {
    pub key: &'static str,
    pub label: &'static str,
    /// 折算到该类别基准单位的倍数，温度不使用它。
    pub factor: f64,
}

#[derive(Clone, Copy)]
struct UnitGroup {
    category: UnitCategory,
    from: &'static str,
    to: &'static str,
    units: &'static [Unit],
}

const fn unit(key: &'static str, factor: f64) -> Unit {
    Unit {
        key,
        label: key,
        factor,
    }
}

const GROUPS: &[UnitGroup] = &[
    UnitGroup {
        category: UnitCategory::Length,
        from: "km",
        to: "mi",
        units: &[
            unit("mm", 0.001),
            unit("cm", 0.01),
            unit("m", 1.0),
            unit("km", 1000.0),
            unit("in", 0.0254),
            unit("ft", 0.3048),
            unit("yd", 0.9144),
            unit("mi", 1609.344),
        ],
    },
    UnitGroup {
        category: UnitCategory::Area,
        from: "m²",
        to: "ft²",
        units: &[
            unit("cm²", 0.0001),
            unit("m²", 1.0),
            unit("km²", 1_000_000.0),
            unit("ha", 10_000.0),
            unit("acre", 4046.856_4224),
            unit("in²", 0.000_645_16),
            unit("ft²", 0.092_903_04),
        ],
    },
    UnitGroup {
        category: UnitCategory::Volume,
        from: "L",
        to: "gal",
        units: &[
            unit("ml", 0.001),
            unit("L", 1.0),
            unit("m³", 1000.0),
            unit("gal", 3.785_411_784),
            unit("fl oz", 0.029_573_529_5625),
            unit("ft³", 28.316_846_592),
        ],
    },
    UnitGroup {
        category: UnitCategory::Mass,
        from: "kg",
        to: "lb",
        units: &[
            unit("g", 0.001),
            unit("kg", 1.0),
            unit("t", 1000.0),
            unit("lb", 0.453_592_37),
            unit("oz", 0.028_349_523_125),
        ],
    },
    UnitGroup {
        category: UnitCategory::Speed,
        from: "km/h",
        to: "mph",
        units: &[
            unit("m/s", 1.0),
            unit("km/h", 1.0 / 3.6),
            unit("mph", 0.447_04),
            unit("kn", 0.514_444_444_444_4445),
            unit("ft/s", 0.3048),
        ],
    },
    UnitGroup {
        category: UnitCategory::Time,
        from: "min",
        to: "s",
        units: &[
            unit("ms", 0.001),
            unit("s", 1.0),
            unit("min", 60.0),
            unit("h", 3600.0),
            unit("day", 86_400.0),
            unit("week", 604_800.0),
        ],
    },
    UnitGroup {
        category: UnitCategory::Temperature,
        from: "°C",
        to: "°F",
        units: &[unit("°C", 1.0), unit("°F", 1.0), unit("K", 1.0)],
    },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConvertKey {
    Digit(char),
    Dot,
    Colon,
    Sign,
    Back,
    Clear,
}

/// 一次换算的产出：空输入、错误，或若干「标签 → 值」。
#[derive(Clone, PartialEq, Debug)]
pub enum Conversion {
    Empty,
    Error(&'static str),
    Rows(Vec<(String, String)>),
}

impl Conversion {
    /// 按标签取某一行结果，只服务于断言之后的界面与测试。
    #[cfg(test)]
    pub fn get(&self, label: &str) -> Option<&str> {
        match self {
            Conversion::Rows(rows) => rows
                .iter()
                .find(|(row, _)| row == label)
                .map(|(_, value)| value.as_str()),
            _ => None,
        }
    }
}

pub struct Converter {
    pub kind: ConvertKind,
    pub base: Base,
    pub category: UnitCategory,
    pub from_unit: &'static str,
    pub to_unit: &'static str,
    buffer: String,
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

impl Converter {
    pub fn new() -> Self {
        Self {
            kind: ConvertKind::Base,
            base: Base::Dec,
            category: UnitCategory::Length,
            from_unit: "km",
            to_unit: "mi",
            buffer: String::new(),
        }
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// 换基数或换类别时清空输入：旧输入在新上下文里往往无意义。
    pub fn set_kind(&mut self, kind: ConvertKind) {
        if self.kind != kind {
            self.kind = kind;
            self.buffer.clear();
        }
    }

    pub fn set_base(&mut self, base: Base) {
        if self.base != base {
            self.base = base;
            self.buffer.clear();
        }
    }

    pub fn set_category(&mut self, category: UnitCategory) {
        if self.category != category {
            self.category = category;
            self.buffer.clear();
            let group = category.group();
            self.from_unit = group.from;
            self.to_unit = group.to;
        }
    }

    pub fn set_from(&mut self, key: &'static str) {
        self.from_unit = key;
    }

    pub fn set_to(&mut self, key: &'static str) {
        self.to_unit = key;
    }

    /// 输入 `−40` 这类负数需要正负号，进制模式只用无符号整数。
    pub fn accepts_sign(&self) -> bool {
        matches!(self.kind, ConvertKind::Angle | ConvertKind::Unit)
    }

    pub fn accepts_colon(&self) -> bool {
        self.kind == ConvertKind::Angle
    }

    pub fn press(&mut self, key: ConvertKey) {
        match key {
            ConvertKey::Clear => self.buffer.clear(),
            ConvertKey::Back => {
                self.buffer.pop();
            }
            ConvertKey::Dot => {
                if !self.buffer.contains('.') && self.buffer.len() < MAX_INPUT_LEN {
                    self.buffer.push('.');
                }
            }
            ConvertKey::Colon => {
                let filled = !self.buffer.is_empty() && !self.buffer.ends_with(':');
                if self.accepts_colon() && filled && self.buffer.matches(':').count() < 2 {
                    self.buffer.push(':');
                }
            }
            ConvertKey::Sign => {
                if self.accepts_sign() {
                    if self.buffer.starts_with('-') {
                        self.buffer.remove(0);
                    } else {
                        self.buffer.insert(0, '-');
                    }
                }
            }
            ConvertKey::Digit(digit) => {
                if self.buffer.len() >= MAX_INPUT_LEN || !digit.is_ascii_alphanumeric() {
                    return;
                }
                if digit.is_ascii_alphabetic() && self.kind != ConvertKind::Base {
                    return;
                }
                if self.kind == ConvertKind::Base && !self.base.accepts(digit) {
                    return;
                }
                // 键盘小写、按键标签大写，缓冲统一显示大写。
                let digit = digit.to_ascii_uppercase();
                self.buffer.push(digit);
            }
        }
    }

    pub fn conversion(&self) -> Conversion {
        if self.buffer.is_empty() {
            return Conversion::Empty;
        }
        match self.kind {
            ConvertKind::Base => base_rows(&self.buffer, self.base),
            ConvertKind::Angle => angle_rows(&self.buffer),
            ConvertKind::Unit => {
                unit_rows(&self.buffer, self.category, self.from_unit, self.to_unit)
            }
        }
    }
}

fn base_rows(src: &str, base: Base) -> Conversion {
    let mut value: u128 = 0;
    for ch in src.chars() {
        let Some(digit) = Base::digit_value(ch) else {
            return Conversion::Error(ERR_INVALID);
        };
        if digit >= base.radix() {
            return Conversion::Error(ERR_INVALID);
        }
        let Some(next) = value
            .checked_mul(base.radix() as u128)
            .and_then(|scaled| scaled.checked_add(digit as u128))
        else {
            return Conversion::Error(ERR_RANGE);
        };
        if next > u64::MAX as u128 {
            return Conversion::Error(ERR_RANGE);
        }
        value = next;
    }
    let value = value as u64;
    let bits = if value == 0 {
        1
    } else {
        64 - value.leading_zeros()
    };
    Conversion::Rows(vec![
        ("十进制".to_string(), value.to_string()),
        ("十六进制".to_string(), format!("{value:X}")),
        ("八进制".to_string(), format!("{value:o}")),
        ("二进制".to_string(), format!("{value:b}")),
        ("位宽".to_string(), format!("{bits} bit")),
    ])
}

fn angle_rows(src: &str) -> Conversion {
    let Some(degrees) = parse_angle(src) else {
        return Conversion::Error(ERR_INVALID);
    };
    Conversion::Rows(vec![
        ("十进制度".to_string(), format_value(degrees)),
        ("度分秒".to_string(), to_dms(degrees)),
        ("弧度".to_string(), format_value(degrees.to_radians())),
        ("百分度".to_string(), format_value(degrees * 400.0 / 360.0)),
    ])
}

/// `40.446` 与 `40:26:46.2` 都接受：前者是十进制度，后者是度分秒。
fn parse_angle(src: &str) -> Option<f64> {
    if src.contains(':') {
        let parts: Vec<&str> = src.split(':').collect();
        if parts.len() != 3 {
            return None;
        }
        let degrees: f64 = parts[0].parse().ok()?;
        let minutes: f64 = parts[1].parse().ok()?;
        let seconds: f64 = parts[2].parse().ok()?;
        if !degrees.is_finite()
            || minutes < 0.0
            || seconds < 0.0
            || minutes >= 60.0
            || seconds >= 60.0
        {
            return None;
        }
        let sign = if degrees < 0.0 { -1.0 } else { 1.0 };
        Some(sign * (degrees.abs() + minutes / 60.0 + seconds / 3600.0))
    } else {
        src.parse::<f64>().ok().filter(|value| value.is_finite())
    }
}

fn to_dms(degrees: f64) -> String {
    let sign = if degrees < 0.0 { "−" } else { "" };
    let absolute = degrees.abs();
    let mut degree = absolute.floor();
    let mut minutes = (absolute - degree) * 60.0;
    let mut minute = minutes.floor();
    minutes = (minutes - minute) * 60.0;
    let mut second = (minutes * 1e4).round() / 1e4;
    if second >= 60.0 {
        second -= 60.0;
        minute += 1.0;
    }
    if minute >= 60.0 {
        minute -= 60.0;
        degree += 1.0;
    }
    format!(
        "{sign}{}° {}′ {}″",
        degree as i64,
        minute as i64,
        trim_fraction(second)
    )
}

/// 最多四位小数，去掉尾部的 0。
fn trim_fraction(value: f64) -> String {
    let text = format!("{value:.4}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

fn unit_rows(src: &str, category: UnitCategory, from: &str, to: &str) -> Conversion {
    let Ok(value) = src.parse::<f64>() else {
        return Conversion::Error(ERR_INVALID);
    };
    if !value.is_finite() {
        return Conversion::Error(ERR_INVALID);
    }
    let units = category.units();
    let (Some(source), Some(target)) = (find_unit(units, from), find_unit(units, to)) else {
        return Conversion::Error(ERR_INVALID);
    };
    let converted = if category == UnitCategory::Temperature {
        from_celsius(to_celsius(value, source.label), target.label)
    } else {
        value * source.factor / target.factor
    };
    if !converted.is_finite() {
        return Conversion::Error(ERR_RANGE);
    }
    let mut rows = vec![(
        "换算结果".to_string(),
        format!("{} {}", format_value(converted), target.label),
    )];
    if category != UnitCategory::Temperature {
        rows.push((
            "倍率".to_string(),
            format!(
                "1 {} = {} {}",
                source.label,
                format_value(source.factor / target.factor),
                target.label
            ),
        ));
    }
    Conversion::Rows(rows)
}

fn find_unit<'a>(units: &'a [Unit], key: &str) -> Option<&'a Unit> {
    units.iter().find(|unit| unit.key == key)
}

fn to_celsius(value: f64, unit: &str) -> f64 {
    match unit {
        "°F" => (value - 32.0) * 5.0 / 9.0,
        "K" => value - 273.15,
        _ => value,
    }
}

fn from_celsius(celsius: f64, unit: &str) -> f64 {
    match unit {
        "°F" => celsius * 9.0 / 5.0 + 32.0,
        "K" => celsius + 273.15,
        _ => celsius,
    }
}

#[cfg(test)]
mod tests {
    use super::{Base, Conversion, ConvertKey, ConvertKind, Converter, UnitCategory, CATEGORIES};

    fn converted(kind: ConvertKind, base: Base, input: &str, label: &str) -> Option<String> {
        let mut converter = Converter::new();
        converter.set_kind(kind);
        if kind == ConvertKind::Base {
            converter.set_base(base);
        }
        for ch in input.chars() {
            let key = match ch {
                '.' => ConvertKey::Dot,
                ':' => ConvertKey::Colon,
                '-' => ConvertKey::Sign,
                other => ConvertKey::Digit(other),
            };
            converter.press(key);
        }
        converter.conversion().get(label).map(str::to_string)
    }

    #[test]
    fn base_conversions_agree_across_radixes() {
        assert_eq!(
            converted(ConvertKind::Base, Base::Dec, "255", "十六进制").as_deref(),
            Some("FF")
        );
        assert_eq!(
            converted(ConvertKind::Base, Base::Hex, "ff", "十进制").as_deref(),
            Some("255")
        );
        assert_eq!(
            converted(ConvertKind::Base, Base::Bin, "1011", "十进制").as_deref(),
            Some("11")
        );
        assert_eq!(
            converted(ConvertKind::Base, Base::Dec, "8", "二进制").as_deref(),
            Some("1000")
        );
        assert_eq!(
            converted(ConvertKind::Base, Base::Dec, "0", "位宽").as_deref(),
            Some("1 bit")
        );
        assert_eq!(
            converted(ConvertKind::Base, Base::Dec, "255", "位宽").as_deref(),
            Some("8 bit")
        );
    }

    #[test]
    fn base_input_rejects_bad_digits_and_overflow() {
        let mut converter = Converter::new();
        converter.set_kind(ConvertKind::Base);
        converter.set_base(Base::Dec);
        converter.press(ConvertKey::Digit('9'));
        converter.press(ConvertKey::Digit('f'));
        assert_eq!(converter.buffer(), "9");

        converter.set_base(Base::Hex);
        assert_eq!(converter.buffer(), "");
        for digit in "ffffffffffffffff".chars() {
            converter.press(ConvertKey::Digit(digit));
        }
        assert_eq!(
            converter.conversion().get("十进制"),
            Some("18446744073709551615"),
            "u64 上限本身仍可表示"
        );
        converter.press(ConvertKey::Digit('1'));
        assert_eq!(
            converter.conversion(),
            Conversion::Error("数值超出范围"),
            "超过 u64 后溢出可见"
        );
        converter.press(ConvertKey::Clear);
        assert_eq!(converter.conversion(), Conversion::Empty);
    }

    #[test]
    fn decimal_degrees_and_dms_convert_both_ways() {
        assert_eq!(
            converted(ConvertKind::Angle, Base::Dec, "40.44617", "度分秒").as_deref(),
            Some("40° 26′ 46.212″")
        );
        assert_eq!(
            converted(ConvertKind::Angle, Base::Dec, "40:26:46.212", "十进制度").as_deref(),
            Some("40.44617")
        );
        assert_eq!(
            converted(ConvertKind::Angle, Base::Dec, "-40:26:46.212", "度分秒").as_deref(),
            Some("−40° 26′ 46.212″")
        );
        assert_eq!(
            converted(ConvertKind::Angle, Base::Dec, "360", "百分度").as_deref(),
            Some("400")
        );
        assert_eq!(
            converted(ConvertKind::Angle, Base::Dec, "45", "弧度").as_deref(),
            Some("0.785398163397")
        );
    }

    #[test]
    fn dms_rounds_up_without_leaving_60_seconds() {
        assert_eq!(
            converted(ConvertKind::Angle, Base::Dec, "10:59:59.99996", "度分秒").as_deref(),
            Some("11° 0′ 0″")
        );
    }

    #[test]
    fn angle_input_rejects_malformed_timecode() {
        let mut converter = Converter::new();
        converter.set_kind(ConvertKind::Angle);
        converter.press(ConvertKey::Colon);
        assert_eq!(converter.buffer(), "", "空输入不接受冒号");
        converter.press(ConvertKey::Digit('4'));
        converter.press(ConvertKey::Colon);
        converter.press(ConvertKey::Colon);
        assert_eq!(converter.buffer(), "4:", "连续冒号被忽略");
        assert_eq!(converter.conversion(), Conversion::Error("表达式无效"));
    }

    #[test]
    fn linear_units_use_exact_factors() {
        let mut converter = Converter::new();
        converter.set_kind(ConvertKind::Unit);
        converter.press(ConvertKey::Digit('1'));
        converter.press(ConvertKey::Digit('0'));
        converter.press(ConvertKey::Digit('0'));
        assert_eq!(
            converter.conversion().get("换算结果"),
            Some("62.1371192237 mi"),
            "默认长度换算 km → mi"
        );
        assert_eq!(
            converter.conversion().get("倍率"),
            Some("1 km = 0.621371192237 mi")
        );
        converter.set_category(UnitCategory::Time);
        assert_eq!(converter.buffer(), "");
        converter.press(ConvertKey::Digit('9'));
        converter.press(ConvertKey::Digit('0'));
        assert_eq!(
            converter.conversion().get("换算结果"),
            Some("5400 s"),
            "切换到时间后默认 min → s"
        );
    }

    #[test]
    fn temperature_is_affine_not_linear() {
        let mut converter = Converter::new();
        converter.set_kind(ConvertKind::Unit);
        converter.set_category(UnitCategory::Temperature);
        for digit in "-40".chars() {
            converter.press(if digit == '-' {
                ConvertKey::Sign
            } else {
                ConvertKey::Digit(digit)
            });
        }
        assert_eq!(
            converter.conversion().get("换算结果"),
            Some("-40 °F"),
            "-40 °C 与 -40 °F 相同"
        );
        assert_eq!(converter.conversion().get("倍率"), None, "温度没有倍率");
        converter.set_to("K");
        converter.set_from("°F");
        converter.press(ConvertKey::Clear);
        for digit in "212".chars() {
            converter.press(ConvertKey::Digit(digit));
        }
        assert_eq!(converter.conversion().get("换算结果"), Some("373.15 K"));
    }

    #[test]
    fn letters_are_ignored_outside_hex() {
        let mut converter = Converter::new();
        converter.set_kind(ConvertKind::Angle);
        converter.press(ConvertKey::Digit('4'));
        converter.press(ConvertKey::Digit('a'));
        assert_eq!(converter.buffer(), "4");

        converter.set_kind(ConvertKind::Base);
        converter.press(ConvertKey::Digit('a'));
        assert_eq!(converter.buffer(), "", "十进制下字母无效");
        converter.set_base(Base::Hex);
        converter.press(ConvertKey::Digit('A'));
        assert_eq!(converter.buffer(), "A");
        assert_eq!(converter.conversion().get("十进制"), Some("10"));
    }

    #[test]
    fn every_category_can_convert() {
        for category in CATEGORIES {
            let mut converter = Converter::new();
            converter.set_kind(ConvertKind::Unit);
            converter.set_category(category);
            converter.press(ConvertKey::Digit('1'));
            assert!(
                matches!(converter.conversion(), Conversion::Rows(_)),
                "{:?} 应能换算",
                category
            );
            assert!(category
                .units()
                .iter()
                .any(|unit| unit.key == converter.from_unit));
            assert!(category.units().len() >= 3, "{:?} 至少三个单位", category);
        }
    }
}
