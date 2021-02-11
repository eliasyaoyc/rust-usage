use crate::classpath::Entry;

pub struct ClassPath {
    pub boot_classpath: Box<dyn Entry>,
    pub ext_classpath: Box<dyn Entry>,
    pub user_classpath: Box<dyn Entry>,
}

impl ClassPath {
    pub fn parse(jre_option: String, cp_option: String) -> ClassPath {

    }
}