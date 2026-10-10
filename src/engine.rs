//! 表达式解析、求值与计算器状态机。

/// 三角函数的角度单位。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Angle {
    Deg,
    Rad,
}

impl Angle {
    pub fn label(self) -> &'static str {
        match self {
            Angle::Deg => "DEG",
            Angle::Rad => "RAD",
        }
    }

    fn to_rad(self, x: f64) -> f64 {
        match self {
            Angle::Deg => x.to_radians(),
            Angle::Rad => x,
        }
    }

    fn from_rad(self, x: f64) -> f64 {
        match self {
            Angle::Deg => x.to_degrees(),
            Angle::Rad => x,
        }
    }
}

pub const ERR_INVALID: &str = "表达式无效";
pub const ERR_DIV_ZERO: &str = "除数为零";
pub const ERR_DOMAIN: &str = "输入超出定义域";
pub const ERR_RANGE: &str = "数值超出范围";

/// 一个按键输入，UI 与物理键盘都映射到这里。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Digit(char),
    Dot,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    Percent,
    Factorial,
    LParen,
    RParen,
    Sin,
    Cos,
    Tan,
    ASin,
    ACos,
    ATan,
    Ln,
    Log,
    Log2,
    Exp,
    Abs,
    Sqrt,
    Cbrt,
    Square,
    Cube,
    Reciprocal,
    Pi,
    E,
    Answer,
    Negate,
    Backspace,
    Clear,
    ClearHistory,
    Equals,
    ToggleAngle,
    SetMode(Mode),
}

impl Key {
    /// 插入到表达式里的文本；数字单独处理。
    fn text(self) -> Option<&'static str> {
        Some(match self {
            Key::Dot => ".",
            Key::Add => "+",
            Key::Sub => "-",
            Key::Mul => "×",
            Key::Div => "÷",
            Key::Pow => "^",
            Key::Mod => " mod ",
            Key::Percent => "%",
            Key::Factorial => "!",
            Key::LParen => "(",
            Key::RParen => ")",
            Key::Sin => "sin(",
            Key::Cos => "cos(",
            Key::Tan => "tan(",
            Key::ASin => "asin(",
            Key::ACos => "acos(",
            Key::ATan => "atan(",
            Key::Ln => "ln(",
            Key::Log => "log(",
            Key::Log2 => "log2(",
            Key::Exp => "exp(",
            Key::Abs => "abs(",
            Key::Sqrt => "√(",
            Key::Cbrt => "∛(",
            Key::Square => "^2",
            Key::Cube => "^3",
            Key::Reciprocal => "^-1",
            Key::Pi => "π",
            Key::E => "e",
            _ => return None,
        })
    }

    /// 该键是否可以开启一条全新输入（用于等号之后继续按数字）。
    fn starts_new_entry(self) -> bool {
        matches!(
            self,
            Key::Digit(_)
                | Key::Dot
                | Key::LParen
                | Key::Negate
                | Key::Pi
                | Key::E
                | Key::Answer
                | Key::Sin
                | Key::Cos
                | Key::Tan
                | Key::ASin
                | Key::ACos
                | Key::ATan
                | Key::Ln
                | Key::Log
                | Key::Log2
                | Key::Exp
                | Key::Abs
                | Key::Sqrt
                | Key::Cbrt
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Standard,
    Scientific,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Mode::Standard => "标准",
            Mode::Scientific => "科学",
        }
    }
}

/// 计算器状态：表达式文本、上次结果、历史记录与错误。
pub struct Calculator {
    pub mode: Mode,
    pub angle: Angle,
    expr: String,
    result: Option<String>,
    error: Option<&'static str>,
    history: Vec<(String, String)>,
    just_evaluated: bool,
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            mode: Mode::Standard,
            angle: Angle::Deg,
            expr: String::new(),
            result: None,
            error: None,
            history: Vec::new(),
            just_evaluated: false,
        }
    }

    pub fn expr(&self) -> &str {
        &self.expr
    }

    pub fn result(&self) -> Option<&str> {
        self.result.as_deref()
    }

    pub fn error(&self) -> Option<&'static str> {
        self.error
    }

    pub fn history(&self) -> &[(String, String)] {
        &self.history
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn angle(&self) -> Angle {
        self.angle
    }

    /// 输入过程中实时求值，失败时返回 None（不显示为错误）。
    pub fn preview(&self) -> Option<String> {
        if self.just_evaluated || self.expr.is_empty() {
            return None;
        }
        eval(&close_open_parens(&self.expr), self.angle)
            .ok()
            .map(format_value)
    }

    pub fn press(&mut self, key: Key) {
        self.error = None;

        match key {
            Key::Clear => {
                self.expr.clear();
                self.result = None;
                self.just_evaluated = false;
                return;
            }
            Key::Equals => return self.equals(),
            Key::Backspace => return self.backspace(),
            Key::Negate => {
                self.just_evaluated = false;
                return self.negate();
            }
            Key::ToggleAngle => {
                self.angle = match self.angle {
                    Angle::Deg => Angle::Rad,
                    Angle::Rad => Angle::Deg,
                };
                return;
            }
            Key::SetMode(mode) => {
                self.mode = mode;
                return;
            }
            Key::ClearHistory => {
                self.history.clear();
                return;
            }
            _ => {}
        }

        if self.just_evaluated && key.starts_new_entry() {
            self.expr.clear();
        }
        if self.expr.is_empty() && !starts_an_expression(key) {
            return;
        }
        if key == Key::Dot && trailing_operand(&self.expr).contains('.') {
            return;
        }

        let token = match key {
            Key::Digit(digit) => digit.to_string(),
            Key::Answer => match self.history.last() {
                Some((_, result)) => result.clone(),
                None => return,
            },
            other => other.text().unwrap_or_default().to_string(),
        };

        self.just_evaluated = false;
        self.expr.push_str(&token);
    }

    fn equals(&mut self) {
        if self.expr.is_empty() {
            return;
        }
        let closed = close_open_parens(&self.expr);
        match eval(&closed, self.angle) {
            Ok(value) => {
                let text = format_value(value);
                self.history.push((closed, text.clone()));
                if self.history.len() > 99 {
                    self.history.remove(0);
                }
                self.expr = text.clone();
                self.result = Some(text);
                self.just_evaluated = true;
            }
            Err(err) => self.error = Some(err),
        }
    }

    /// 删除一个整体 token，函数名不会被拆成单字符。
    fn backspace(&mut self) {
        self.just_evaluated = false;
        if self.expr.is_empty() {
            return;
        }
        for token in TOKENS {
            if self.expr.ends_with(token) {
                self.expr.truncate(self.expr.len() - token.len());
                return;
            }
        }
        let cut = self.expr.len() - self.expr.chars().next_back().map_or(0, |c| c.len_utf8());
        self.expr.truncate(cut);
    }

    /// 对末尾的数字（或常量）切换正负。
    fn negate(&mut self) {
        let Some(start) = trailing_operand_start(&self.expr) else {
            return;
        };
        let prefix = &self.expr[..start];
        if prefix.ends_with('-') && !is_binary_operator_context(prefix) {
            self.expr.replace_range(prefix.len() - 1..start, "");
        } else {
            self.expr.insert_str(start, "-");
        }
    }
}

fn starts_an_expression(key: Key) -> bool {
    !matches!(
        key,
        Key::Add
            | Key::Mul
            | Key::Div
            | Key::Pow
            | Key::Mod
            | Key::Percent
            | Key::Factorial
            | Key::RParen
            | Key::Square
            | Key::Cube
            | Key::Reciprocal
    )
}

/// 整体删除的 token，长 token 必须排在前面。
const TOKENS: &[&str] = &[
    "asin(", "acos(", "atan(", "sinh(", "log2(", "sqrt(", "cbrt(", "sin(", "cos(", "tan(", "abs(",
    "exp(", "log(", "ln(", " mod ", "^2", "^3", "^-1", "√(", "∛(", "π",
];

fn trailing_operand(expr: &str) -> String {
    expr.chars()
        .rev()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

/// 末尾操作数的起始字节位置；末尾不是数字或常量时返回 None。
fn trailing_operand_start(expr: &str) -> Option<usize> {
    let bytes = expr.as_bytes();
    let mut i = bytes.len();
    while i > 0 && (bytes[i - 1].is_ascii_digit() || bytes[i - 1] == b'.') {
        i -= 1;
    }
    if i < bytes.len() {
        return Some(i);
    }
    if expr.ends_with('π') {
        return Some(i - 'π'.len_utf8());
    }
    if i > 0 && bytes[i - 1] == b'e' && !expr[..i - 1].ends_with(char::is_alphabetic) {
        return Some(i - 1);
    }
    None
}

/// 判断前缀末尾的 `-` 是减号（true）还是一元负号（false）。
fn is_binary_operator_context(prefix: &str) -> bool {
    let mut chars = prefix.chars().rev().filter(|c| !c.is_whitespace());
    if chars.next() != Some('-') {
        return false;
    }
    match chars.next() {
        None => false,
        Some(c) => !matches!(c, '(' | '+' | '-' | '×' | '÷' | '*' | '/' | '^' | '%'),
    }
}

/// 把显示层使用的符号转换成解析器认识的写法。
pub fn normalize(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for c in src.chars() {
        match c {
            '×' => out.push('*'),
            '÷' => out.push('/'),
            '−' => out.push('-'),
            'π' => out.push_str("pi"),
            '√' => out.push_str("sqrt"),
            '∛' => out.push_str("cbrt"),
            _ => out.push(c),
        }
    }
    out
}

/// 补全未闭合的括号，让 "sin(30" 这样的输入可以直接求值。
pub fn close_open_parens(src: &str) -> String {
    let mut open = 0usize;
    for c in src.chars() {
        match c {
            '(' => open += 1,
            ')' => open = open.saturating_sub(1),
            _ => {}
        }
    }
    let mut out = src.to_string();
    for _ in 0..open {
        out.push(')');
    }
    out
}

struct Parser {
    src: Vec<char>,
    pos: usize,
    angle: Angle,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.src.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(|c| c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn eat(&mut self, c: char) -> bool {
        self.skip_ws();
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, c: char) -> Result<(), &'static str> {
        if self.eat(c) {
            Ok(())
        } else {
            Err(ERR_INVALID)
        }
    }

    fn word(&mut self, word: &str) -> bool {
        self.skip_ws();
        let rest: String = self.src[self.pos..].iter().collect();
        if !rest.starts_with(word) {
            return false;
        }
        let after = self.src[self.pos + word.len()..].first().copied();
        if after.is_some_and(|c| c.is_ascii_alphanumeric()) {
            return false;
        }
        self.pos += word.len();
        true
    }

    fn parse_all(&mut self) -> Result<f64, &'static str> {
        let value = self.expr()?;
        self.skip_ws();
        if self.pos != self.src.len() {
            return Err(ERR_INVALID);
        }
        finite(value)
    }

    fn expr(&mut self) -> Result<f64, &'static str> {
        let mut acc = self.term()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('+') => {
                    self.pos += 1;
                    acc = finite(acc + self.term()?)?;
                }
                Some('-') => {
                    self.pos += 1;
                    acc = finite(acc - self.term()?)?;
                }
                _ => break,
            }
        }
        Ok(acc)
    }

    fn term(&mut self) -> Result<f64, &'static str> {
        let mut acc = self.unary()?;
        loop {
            self.skip_ws();
            if self.word("mod") {
                acc = checked_mod(acc, self.unary()?)?;
                continue;
            }
            match self.peek() {
                Some('*') => {
                    self.pos += 1;
                    acc = finite(acc * self.unary()?)?;
                }
                Some('/') => {
                    self.pos += 1;
                    let rhs = self.unary()?;
                    if rhs == 0.0 {
                        return Err(ERR_DIV_ZERO);
                    }
                    acc = finite(acc / rhs)?;
                }
                // 隐式乘法：2π、3(4)、(2)3、2sin(0)
                Some(c) if c == '(' || c.is_ascii_alphanumeric() => {
                    acc = finite(acc * self.unary()?)?;
                }
                _ => break,
            }
        }
        Ok(acc)
    }

    fn unary(&mut self) -> Result<f64, &'static str> {
        self.skip_ws();
        match self.peek() {
            Some('-') => {
                self.pos += 1;
                Ok(-self.unary()?)
            }
            Some('+') => {
                self.pos += 1;
                self.unary()
            }
            _ => self.power(),
        }
    }

    fn power(&mut self) -> Result<f64, &'static str> {
        let base = self.postfix()?;
        self.skip_ws();
        if self.peek() == Some('^') {
            self.pos += 1;
            return checked_pow(base, self.unary()?);
        }
        Ok(base)
    }

    fn postfix(&mut self) -> Result<f64, &'static str> {
        let mut value = self.primary()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('!') => {
                    self.pos += 1;
                    value = factorial(value)?;
                }
                Some('%') => {
                    self.pos += 1;
                    value = finite(value / 100.0)?;
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn primary(&mut self) -> Result<f64, &'static str> {
        self.skip_ws();
        match self.peek() {
            Some('(') => {
                self.pos += 1;
                let value = self.expr()?;
                self.expect(')')?;
                Ok(value)
            }
            Some(c) if c.is_ascii_digit() || c == '.' => self.number(),
            Some(c) if c.is_ascii_alphabetic() => {
                let name = self.ident();
                if self.peek() == Some('(') {
                    self.pos += 1;
                    let value = self.expr()?;
                    self.expect(')')?;
                    apply_func(&name, value, self.angle)
                } else {
                    apply_const(&name)
                }
            }
            _ => Err(ERR_INVALID),
        }
    }

    fn number(&mut self) -> Result<f64, &'static str> {
        let start = self.pos;
        let mut dots = 0;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.pos += 1;
            } else if c == '.' {
                dots += 1;
                if dots > 1 {
                    return Err(ERR_INVALID);
                }
                self.pos += 1;
            } else {
                break;
            }
        }
        if start == self.pos {
            return Err(ERR_INVALID);
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            let save = self.pos;
            self.pos += 1;
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.pos += 1;
            }
            if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.pos += 1;
                }
            } else {
                self.pos = save;
            }
        }
        self.src[start..self.pos]
            .iter()
            .collect::<String>()
            .parse::<f64>()
            .map_err(|_| ERR_INVALID)
    }

    fn ident(&mut self) -> String {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphabetic() || c.is_ascii_digit())
        {
            self.pos += 1;
        }
        self.src[start..self.pos].iter().collect()
    }
}

fn finite(value: f64) -> Result<f64, &'static str> {
    if value.is_nan() {
        Err(ERR_DOMAIN)
    } else if value.is_infinite() {
        Err(ERR_RANGE)
    } else {
        Ok(value)
    }
}

fn checked_pow(base: f64, exp: f64) -> Result<f64, &'static str> {
    if base < 0.0 && exp.fract() != 0.0 {
        return Err(ERR_DOMAIN);
    }
    finite(base.powf(exp))
}

fn checked_mod(lhs: f64, rhs: f64) -> Result<f64, &'static str> {
    if rhs == 0.0 {
        return Err(ERR_DIV_ZERO);
    }
    finite(lhs % rhs)
}

fn factorial(value: f64) -> Result<f64, &'static str> {
    if value < 0.0 || value.fract() != 0.0 {
        return Err(ERR_DOMAIN);
    }
    if value > 170.0 {
        return Err(ERR_RANGE);
    }
    let mut acc = 1.0;
    let mut i = 2.0;
    while i <= value {
        acc *= i;
        i += 1.0;
    }
    Ok(acc)
}

fn apply_const(name: &str) -> Result<f64, &'static str> {
    match name {
        "pi" => Ok(std::f64::consts::PI),
        "e" => Ok(std::f64::consts::E),
        _ => Err(ERR_INVALID),
    }
}

fn apply_func(name: &str, x: f64, angle: Angle) -> Result<f64, &'static str> {
    let value = match name {
        "sin" => angle.to_rad(x).sin(),
        "cos" => angle.to_rad(x).cos(),
        "tan" => {
            let r = angle.to_rad(x);
            if r.cos() == 0.0 {
                return Err(ERR_DOMAIN);
            }
            r.tan()
        }
        "asin" if x.abs() > 1.0 => return Err(ERR_DOMAIN),
        "asin" => angle.from_rad(x.asin()),
        "acos" if x.abs() > 1.0 => return Err(ERR_DOMAIN),
        "acos" => angle.from_rad(x.acos()),
        "atan" => angle.from_rad(x.atan()),
        "sinh" => x.sinh(),
        "cosh" => x.cosh(),
        "tanh" => x.tanh(),
        "ln" if x <= 0.0 => return Err(ERR_DOMAIN),
        "ln" => x.ln(),
        "log" if x <= 0.0 => return Err(ERR_DOMAIN),
        "log" => x.log10(),
        "log2" if x <= 0.0 => return Err(ERR_DOMAIN),
        "log2" => x.log2(),
        "sqrt" if x < 0.0 => return Err(ERR_DOMAIN),
        "sqrt" => x.sqrt(),
        "cbrt" => x.cbrt(),
        "abs" => x.abs(),
        "exp" => x.exp(),
        "round" => x.round(),
        "floor" => x.floor(),
        "ceil" => x.ceil(),
        _ => return Err(ERR_INVALID),
    };
    finite(value)
}

pub fn eval(src: &str, angle: Angle) -> Result<f64, &'static str> {
    let normalized = normalize(src);
    if normalized.trim().is_empty() {
        return Err(ERR_INVALID);
    }
    Parser {
        src: normalized.chars().collect(),
        pos: 0,
        angle,
    }
    .parse_all()
}

fn trim_zeros(text: &str) -> String {
    if !text.contains('.') {
        return text.to_string();
    }
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

/// 显示用格式化：约 12 位有效数字，过大或过小的值使用科学计数法。
pub fn format_value(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    let abs = value.abs();
    if abs >= 1e12 || abs < 1e-6 {
        let text = format!("{:.9e}", value);
        return match text.split_once('e') {
            Some((mantissa, exponent)) => format!("{}e{}", trim_zeros(mantissa), exponent),
            None => text,
        };
    }
    let digits_before = abs.log10().floor() as i32 + 1;
    let decimals = (12 - digits_before).clamp(0, 12);
    trim_zeros(&format!("{:.*}", decimals as usize, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deg(expr: &str) -> f64 {
        eval(expr, Angle::Deg).unwrap()
    }

    fn rad(expr: &str) -> f64 {
        eval(expr, Angle::Rad).unwrap()
    }

    fn approx(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {actual} to approximate {expected}"
        );
    }

    #[test]
    fn arithmetic_precedence() {
        assert_eq!(deg("2+3*4"), 14.0);
        assert_eq!(deg("(2+3)*4"), 20.0);
        assert_eq!(deg("2^-1"), 0.5);
        assert_eq!(deg("-2^2"), -4.0);
        assert_eq!(deg("2^3^2"), 512.0);
    }

    #[test]
    fn display_glyphs_and_implicit_multiplication() {
        assert_eq!(deg("6×8÷4"), 12.0);
        assert_eq!(rad("2π"), 2.0 * std::f64::consts::PI);
        assert_eq!(deg("√(9)+abs(-2)"), 5.0);
        assert_eq!(deg("3(4)"), 12.0);
    }

    #[test]
    fn functions_and_angle_units() {
        approx(deg("sin(30)"), 0.5);
        approx(rad("sin(0)"), 0.0);
        approx(deg("asin(1)"), 90.0);
        approx(deg("log(1000)"), 3.0);
        approx(deg("5!"), 120.0);
        approx(deg("50%"), 0.5);
        approx(deg("10 mod 3"), 1.0);
    }

    #[test]
    fn errors() {
        assert_eq!(eval("1/0", Angle::Deg), Err(ERR_DIV_ZERO));
        assert_eq!(eval("ln(-1)", Angle::Deg), Err(ERR_DOMAIN));
        assert_eq!(eval("1+", Angle::Deg), Err(ERR_INVALID));
        assert_eq!(eval("2 3+", Angle::Deg), Err(ERR_INVALID));
        assert_eq!(eval("", Angle::Deg), Err(ERR_INVALID));
    }

    #[test]
    fn formatting() {
        assert_eq!(format_value(14.0), "14");
        assert_eq!(format_value(0.5), "0.5");
        assert_eq!(format_value(1.0 / 3.0), "0.333333333333");
        assert_eq!(format_value(1e15), "1e15");
        assert_eq!(format_value(-0.000_000_001), "-1e-9");
    }

    #[test]
    fn state_machine() {
        let mut calc = Calculator::new();
        for key in [
            Key::Digit('7'),
            Key::Mul,
            Key::Digit('8'),
            Key::Equals,
            Key::Add,
            Key::Digit('2'),
            Key::Equals,
        ] {
            calc.press(key);
        }
        assert_eq!(calc.result(), Some("58"));
        assert_eq!(calc.history().len(), 2);

        calc.press(Key::Digit('1'));
        assert_eq!(calc.expr(), "1");

        calc.press(Key::Clear);
        for key in [Key::Digit('5'), Key::Add, Key::Digit('3'), Key::Equals] {
            calc.press(key);
        }
        assert_eq!(calc.expr(), "8");
        calc.press(Key::Negate);
        assert_eq!(calc.expr(), "-8");
        calc.press(Key::Negate);
        assert_eq!(calc.expr(), "8");

        calc.press(Key::Clear);
        for key in [Key::Sin, Key::Digit('3'), Key::Digit('0')] {
            calc.press(key);
        }
        assert_eq!(calc.expr(), "sin(30");
        assert_eq!(calc.preview().as_deref(), Some("0.5"));
        calc.press(Key::Backspace);
        assert_eq!(calc.expr(), "sin(3");
        calc.press(Key::Backspace);
        calc.press(Key::Backspace);
        calc.press(Key::Backspace);
        calc.press(Key::Backspace);
        assert_eq!(calc.expr(), "");
    }

    #[test]
    fn backspace_removes_whole_tokens() {
        let mut calc = Calculator::new();
        calc.press(Key::Sqrt);
        calc.press(Key::Backspace);
        assert_eq!(calc.expr(), "");
        calc.press(Key::Pi);
        calc.press(Key::Backspace);
        assert_eq!(calc.expr(), "");
    }
}
