use image::{DynamicImage, open};
use std::collections::HashMap;

pub struct ServerState {
    pub references: HashMap<i32, DynamicImage>,
    pub user_image: HashMap<String, DynamicImage>,
}

impl ServerState {
    pub fn new() -> Self {
        let mut references: HashMap<i32, DynamicImage> = HashMap::new();
        (0..=9).for_each(|i| {
            let path = format!("assets/{}.bmp", i);
            let image = open(&path).expect("reference must be loaded");
            references.insert(i, image);
        });

        Self {
            references,
            user_image: HashMap::new(),
        }
    }
}
