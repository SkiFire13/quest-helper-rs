use std::{collections::BTreeMap, sync::LazyLock};

use crate::{Client, Runner};

pub static REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::init);

pub struct Registry {
    map: BTreeMap<u32, BTreeMap<u32, &'static Runner>>,
}

impl Registry {
    fn init() -> Self {
        let mut map = BTreeMap::<u32, BTreeMap<u32, &Runner>>::new();
        for runner in crate::RUNNERS {
            let old_entry = map
                .entry(runner.day())
                .or_default()
                .insert(runner.part(), runner);

            assert!(
                old_entry.is_none(),
                "Runner defined twice for day {} and part {}",
                runner.day(),
                runner.part()
            );
        }
        Self { map }
    }

    pub fn get_last_day(&self) -> Option<u32> {
        self.map.last_key_value().map(|(&day, _)| day)
    }

    pub fn run_all(&self, client: &dyn Client, event: &str) {
        for &day in self.map.keys() {
            self.run_day_part_inner(client, event, day, None)
        }
    }

    pub fn run_day(&self, client: &dyn Client, event: &str, day: u32) {
        self.run_day_part_inner(client, event, day, None)
    }

    pub fn run_part(&self, client: &dyn Client, event: &str, day: u32, part: u32) {
        self.run_day_part_inner(client, event, day, Some(part))
    }

    pub fn run_last_day(&self, client: &dyn Client, event: &str) {
        let day = self.get_last_day().expect("No day registered");
        self.run_day_part_inner(client, event, day, None)
    }

    fn run_day_part_inner(
        &self,
        client: &dyn Client,
        event: &str,
        day: u32,
        only_part: Option<u32>,
    ) {
        println!("Day {day:<2}");
        if let Some(parts) = self.map.get(&day) {
            for (&part, runner) in parts {
                if only_part.is_none_or(|only_part| only_part == part) {
                    let Some(input) = client.get_input(event, day, part) else {
                        if let Some(only_part) = only_part {
                            println!("  - Part {only_part}: not available");
                        }
                        continue;
                    };

                    let now = std::time::Instant::now();
                    let res = runner.run(&input);
                    let took = now.elapsed();

                    println!();
                    println!("  - Part {part}: {res}");
                    println!("    Took {took:?}");
                }
            }
        } else {
            println!();
            println!("  - Found no parts registered");
        }
        println!();
    }
}
