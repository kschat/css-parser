use std::ops::Add;


#[derive(Debug)]
pub struct StyleSheet {
    pub rules: Vec<Rule>,
}


#[derive(Debug)]
pub struct Rule {
    pub selectors: Vec<SelectorGroup>,
    pub properties: Vec<Property>,
}


#[derive(Debug)]
pub struct SelectorGroup(pub Vec<Selector>);

impl SelectorGroup {
    pub fn specificity(&self) -> Specificity {
        self.0.iter().fold(
            Specificity::empty(),
            |spec, sel| spec + sel.specificity()
        )
    }
}


#[derive(Debug)]
pub struct Selector {
    pub id: Option<String>,
    pub tag_name: Option<String>,
    pub class_names: Vec<String>,
}

impl Selector {
    pub fn specificity(&self) -> Specificity {
        Specificity(
            if self.id.is_some() { 1 } else { 0 },
            self.class_names.iter().count() as u32,
            if self.tag_name.is_some() { 1 } else { 0 },
        )
    }
}


#[derive(Debug)]
pub struct Specificity(u32, u32, u32);

impl Specificity {
    pub fn empty() -> Specificity {
        Specificity(0, 0, 0)
    }
}

impl Add for Specificity {
    type Output = Specificity;

    fn add(self, rhs: Self) -> Self::Output {
        Specificity(
            self.0 + rhs.0,
            self.1 + rhs.1,
            self.2 + rhs.2,
        )
    }
}


#[derive(Debug)]
pub struct Property {
    pub name: String,
    pub value: DataType,
}


#[derive(Debug)]
pub enum DataType {
    Keyword(String),
    // ColorKeyword(String),
}
