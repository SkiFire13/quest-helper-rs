use std::path::Path;

pub struct SetupInfo {
    pub template: &'static str,
    pub crate_root: &'static str,
}

#[macro_export]
macro_rules! setup_info {
    ($template:literal) => {
        $crate::SetupInfo {
            template: include_str!($template),
            crate_root: env!("CARGO_MANIFEST_DIR"),
        }
    };
}

impl SetupInfo {
    pub(crate) fn write_template(&self, day: u32) {
        let root_path = Path::new(self.crate_root);
        let src_path = root_path.join("src");
        let day_path = src_path.join(format!("day{day}.rs"));

        std::fs::write(day_path, self.template).expect("Failed to write template");

        let main_path = src_path.join("main.rs");
        let mut main_content =
            std::fs::read_to_string(&main_path).expect("Failed to read main file");

        let new_line = format!("mod day{day};");

        if main_content.contains(&new_line) {
            return;
        }

        let insert_pos = if day > 1 {
            let prev_line = format!("mod day{};", day - 1);
            if let Some(i) = main_content.find(&prev_line) {
                i + prev_line.len()
            } else {
                0
            }
        } else {
            0
        };

        main_content.replace_range(insert_pos..insert_pos, &format!("\n{new_line}"));

        std::fs::write(&main_path, &main_content).expect("Failed to update main");
    }
}
