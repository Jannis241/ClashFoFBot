pub struct Settings {
    profile_name: String,
    coc_tag: String,
}

impl Settings {
    pub fn new(profile_name: String, coc_tag: String) -> Self {
        Settings {
            profile_name,
            coc_tag,
        }
    }
    pub fn save(&self) {
        // save to json
    }

    pub fn delete(&self) {}
}

pub fn get_profile(profile_name: &String) -> Settings {
    Settings::new(String::from("test"), String::from("test"))
}

pub fn get_all_profile_names() -> Vec<String> {
    vec![]
}
