use std::path::{Path, PathBuf};

use aes::cipher::{BlockModeDecrypt, KeyIvInit as _, block_padding::Pkcs7};
use serde::Deserialize;
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
    seed: String,
}

impl Client {
    pub fn init(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();

        let token = load_token(&root);
        let agent = build_agent(&token);
        let seed = load_seed(&root, &agent);

        Self { root, agent, seed }
    }

    fn input_path(&self, event: &str, day: u32, part: u32) -> PathBuf {
        self.root
            .join("inputs")
            .join(event)
            .join(day.to_string())
            .join(part.to_string())
    }

    fn write_input(&self, event: &str, day: u32, part: u32, input: &str) {
        let path = self.input_path(event, day, part);
        std::fs::create_dir_all(path.parent().unwrap()).expect("Failed to create input directory");
        std::fs::write(path, input).expect("Failed to write input");
    }

    fn fetch_inputs(&self, event: &str, day: u32) -> Input {
        let seed = &self.seed;
        let input_url = format!("https://everybody.codes/assets/{event}/{day}/input/{seed}.json");
        let aes_url = format!("https://api.everybody.codes/event/{event}/quest/{day}");

        #[derive(Deserialize, Debug)]
        struct InputResponse {
            #[serde(rename = "1")]
            p1: String,
            #[serde(rename = "2")]
            p2: String,
            #[serde(rename = "3")]
            p3: String,
        }

        let input = self
            .agent
            .get(input_url)
            .call()
            .expect("Failed to fetch input")
            .body_mut()
            .read_json::<InputResponse>()
            .expect("Failed to read input response");

        #[derive(Deserialize, Debug)]
        pub struct KeysResponse {
            key1: String,
            key2: Option<String>,
            key3: Option<String>,
        }

        let keys = self
            .agent
            .get(aes_url)
            .call()
            .expect("Failed to fetch AES keys")
            .body_mut()
            .read_json::<KeysResponse>()
            .expect("Failed to read AES keys");

        let input = Input {
            p1: decrypt(&input.p1, &keys.key1),
            p2: keys.key2.map(|key2| decrypt(&input.p2, &key2)),
            p3: keys.key3.map(|key3| decrypt(&input.p3, &key3)),
        };

        self.write_input(event, day, 1, &input.p1);
        if let Some(p2) = &input.p2 {
            self.write_input(event, day, 2, p2);
        }
        if let Some(p3) = &input.p3 {
            self.write_input(event, day, 3, p3);
        }

        input
    }
}

impl quest_runner::Client for Client {
    fn get_input(&self, event: &str, day: u32, part: u32) -> Option<String> {
        let input_path = self.input_path(event, day, part);
        if let Some(input) = std::fs::read_to_string(input_path).ok() {
            return Some(input);
        }

        let inputs = self.fetch_inputs(event, day);

        let input = match part {
            1 => Some(inputs.p1),
            2 => inputs.p2,
            3 => inputs.p3,
            _ => panic!("Invalid part"),
        };

        input
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

    let uri = Uri::from_static("https://api.everybody.codes");
    let cookie =
        Cookie::parse(format!("everybody-codes={token};"), &uri).expect("Failed to parse cookie");
    agent
        .cookie_jar_lock()
        .insert(cookie, &uri)
        .expect("Failed to insert cookie");

    agent
}

fn load_seed(root: &Path, agent: &Agent) -> String {
    let seed_path = root.join(".seed");

    if let Ok(seed) = std::fs::read_to_string(&seed_path) {
        return seed.trim().to_string();
    }

    #[derive(Deserialize)]
    struct SeedResponse {
        seed: u64,
    }

    let seed = agent
        .get("https://api.everybody.codes/user/me")
        .call()
        .expect("Failed to fetch seed")
        .body_mut()
        .read_json::<SeedResponse>()
        .expect("Failed to read seed response")
        .seed
        .to_string();

    if let Err(e) = std::fs::write(seed_path, &seed) {
        eprint!("Failed to store seed: {e}");
    }

    seed
}

struct Input {
    p1: String,
    p2: Option<String>,
    p3: Option<String>,
}

fn decrypt(input: &str, key: &str) -> String {
    type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

    let ciphertext = hex::decode(input).expect("Failed to hex-decode input");

    // Java's "AES/CBC/PKCS5Padding": 32-byte key, 16-byte IV (the first 16
    // bytes of the key). PKCS#5 and PKCS#7 are identical for a 16-byte block.
    let cipher = Aes256CbcDec::new_from_slices(key.as_bytes(), &key.as_bytes()[..16])
        .expect("Invalid AES key/iv length");

    let plaintext = cipher
        .decrypt_padded_vec::<Pkcs7>(&ciphertext)
        .expect("Failed to decrypt");

    String::from_utf8_lossy(&plaintext).into_owned()
}
