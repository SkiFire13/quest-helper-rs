#[macro_export]
macro_rules! register_part {
    ($f:ident) => {{
        #[$crate::__linkme::distributed_slice($crate::RUNNERS)]
        #[linkme(crate = $crate::__linkme)]
        static RUNNERS: $crate::Runner =
            $crate::Runner::new(::std::module_path!(), ::std::stringify!($f), |input| {
                $f(input).to_string()
            });
    }};
}

pub struct Runner {
    day: u32,
    part: u32,
    f: fn(&str) -> String,
}

impl Runner {
    pub const fn new(module: &'static str, fn_name: &'static str, f: fn(&str) -> String) -> Self {
        Self {
            day: extract_day(module),
            part: extract_part(fn_name),
            f,
        }
    }

    pub fn day(&self) -> u32 {
        self.day
    }

    pub fn part(&self) -> u32 {
        self.part
    }

    pub fn run(&self, input: &str) -> String {
        (self.f)(input)
    }
}

const fn extract_trailing_number(s: &'static str) -> &'static str {
    let bytes = s.as_bytes();
    let mut i = bytes.len();

    while i != 0 && matches!(bytes[i - 1], b'0'..=b'9') {
        i -= 1;
    }

    let (_, num) = s.split_at(i);
    num
}

const fn extract_day(module: &'static str) -> u32 {
    parse_num(extract_trailing_number(module))
}

const fn extract_part(fn_name: &'static str) -> u32 {
    parse_num(extract_trailing_number(fn_name))
}

const fn parse_num(part: &'static str) -> u32 {
    let bytes = part.as_bytes();
    let mut n = 0;
    let mut i = 0;

    while i < bytes.len() {
        n = 10 * n + (bytes[i] - b'0') as u32;
        i += 1;
    }

    n
}
