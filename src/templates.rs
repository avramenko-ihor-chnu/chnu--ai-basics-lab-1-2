#[derive(askama::Template)]
#[template(path = "image-analysis-table.html")]
pub struct Table<'a> {
    pub uuid: &'a str,
    pub rows: Vec<Row>,
    pub unique_numbers: Vec<u32>,
}

pub struct Row {
    pub guess: usize,
    pub cells: u32,
    pub diff: u32,
}

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexHtml<'a> {
    pub uuid: &'a str,
}
