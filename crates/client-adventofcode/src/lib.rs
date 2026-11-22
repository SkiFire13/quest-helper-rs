// TODO: adventofcode client

use std::path::{Path, PathBuf};

use ureq::{Agent, Cookie, http::Uri};

#[macro_export]
macro_rules! init_client {
    () => {{
        static CLIENT: ::std::sync::LazyLock<$crate::Client> =
            ::std::sync::LazyLock::new(|| $crate::Client::init(env!("CARGO_MANIFEST_DIR")));

        &*CLIENT
    }};
}

pub struct Client {
    root: PathBuf,
    agent: Agent,
}

impl Client {
    pub fn init(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();

        let token = load_token(&root);
        let agent = build_agent(&token);

        Self { root, agent }
    }

    fn input_path(&self, event: &str, day: u32) -> PathBuf {
        self.root.join("inputs").join(event).join(day.to_string())
    }

    fn write_input(&self, event: &str, day: u32, input: &str) {
        let path = self.input_path(event, day);
        std::fs::create_dir_all(path.parent().unwrap()).expect("Failed to create input directory");
        std::fs::write(path, input).expect("Failed to write input");
    }

    fn fetch_input(&self, event: &str, day: u32) -> Option<String> {
        let timezone = jiff::tz::TimeZone::get("America/New_York")
            .unwrap_or_else(|_| jiff::tz::TimeZone::fixed(jiff::tz::offset(-5)));
        let today = jiff::Timestamp::now().to_zoned_datetime(timezone).date();
        let min_datetime =
            jiff::civil::date(event.parse::<i16>().expect("Invalid year"), 12, day as i8);
        if today < min_datetime {
            return None;
        }

        let url = format!("https://adventofcode.com/{}/day/{}/input", event, day);
        let body = self
            .agent
            .get(&url)
            .call()
            .expect("Input request failed")
            .body_mut()
            .read_to_string()
            .expect("Input request failed");

        if body == "Puzzle inputs differ by user.  Please log in to get your puzzle input.\n" {
            panic!("Invalid session cookie");
        }

        if body.starts_with("Please don't repeatedly request this endpoint before it unlocks!") {
            return None;
        }

        self.write_input(event, day, &body);

        Some(body)
    }
}

impl quest_runner::Client for Client {
    fn get_input(&self, event: &str, day: u32, _part: u32) -> Option<String> {
        let input_path = self.input_path(event, day);
        if let Some(input) = std::fs::read_to_string(input_path).ok() {
            return Some(input);
        }

        self.fetch_input(event, day)
    }
}

fn load_token(root: &Path) -> String {
    std::fs::read_to_string(root.join(".token"))
        .expect("Failed to read token")
        .trim()
        .to_string()
}

fn build_agent(token: &str) -> Agent {
    let agent = Agent::new_with_defaults();

    let uri = Uri::from_static("https://adventofcode.com");
    let cookie = Cookie::parse(format!("session={token};"), &uri).expect("Failed to parse cookie");
    agent
        .cookie_jar_lock()
        .insert(cookie, &uri)
        .expect("Failed to insert cookie");

    agent
}
