mod composite_entry;
mod classpath;
mod zip_entry;
mod wild_card;

pub trait Entry {
    fn read_class(&self, class_name: String) -> Vec<u8>;
    fn to_string(&self) -> String;
}