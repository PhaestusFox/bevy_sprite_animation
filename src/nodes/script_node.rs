use crate::{prelude::{NodeID, Attribute, AnimationNode, NodeResult}, state::AnimationState};

impl AnimationNode for ScriptNode {
    fn run(&self, state: &mut crate::state::AnimationState) -> NodeResult {
        let mut index = 0;
        while index < self.tokens.len() {
            match self.tokens[index] {
                Token::If => {
                    if if_condishion(state, &self.tokens[index + 1], &self.tokens[index + 2], &self.tokens[index + 3]) {
                        index += 4;
                    } else {
                        index += 7;
                    }
                }
                Token::Set => {
                    let key = match self.tokens[index + 1] {
                        Token::Index(i) => i,
                        Token::Attribute(i) => i,
                        _ => {panic!("unimplemented `set {:?}`", self.tokens[index + 1])}
                    };
                    if let Token::Raw(v) = &self.tokens[index + 2] {
                        let data = state.get_attribute_raw_mut(key);
                        *data = v.clone();
                    } else {panic!("unimplemented `set {:?} = {:?}`", key, self.tokens[index + 2])};
                    index += 3;
                },
                Token::Return(id) => {
                    return NodeResult::Next(id);
                }
                _ => todo!("Token {:?}", self.tokens[index])
            }
            index += 1;
        }
        if let Some(fallback) = self.fallback {
            NodeResult::Next(fallback)
        }
        else {
            NodeResult::Error("ScriptNode: failed to find a node to return and no fallback was set;\n
            use #fallback followed by a NodeID at the begging of you script to set a fallback node".to_string())
        }
    }

    fn name(&self) -> &str {
        for tag in self.tags.iter() {
            if let Tag::Name(name) = tag {
                return name;
            }
        }
        "unnamed stript; add #name to the first line to add a name"
    }

    fn ui(&mut self, _ui: &mut bevy_inspector_egui::egui::Ui, _context: &mut bevy_inspector_egui::Context) -> bool {
        false
    }

    fn node_type(&self) -> String {
        "ScriptNode".to_string()
    }

    fn hash(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for tag in self.tags.iter() {
            Hash::hash(tag, &mut hasher);
        }
        for token in self.tokens.iter() {
            Hash::hash(token, &mut hasher);
        }
        hasher.finish()
    }

    fn id(&self) -> NodeID {
        let mut has_name = None;
        for tag in self.tags.iter() {
            if let Tag::Name(name) = tag {
                has_name = Some(name);
                break;
            }
            if let Tag::ID(id) = tag {
                return *id;
            }
        }
        if let Some(name) = has_name {
            NodeID::from_str(name)
        } else {
            use std::hash::Hash;
            use std::hash::Hasher;
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            for tag in self.tags.iter() {
                Hash::hash(tag, &mut hasher);
            }
            for token in self.tokens.iter() {
                Hash::hash(token, &mut hasher);
            }
            NodeID::from_u64(hasher.finish())
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum Token {
    Int(usize),
    Float(usize),
    String(String),
    Raw(Vec<u8>),
    Plus,
    Minus,
    Equals,
    NotEquals,
    LessThen,
    LestThenEq,
    GratterThen,
    GratterThenEq,
    Set,
    Attribute(Attribute),
    Index(Attribute),
    NodeID(NodeID),
    OpenParen(u8),
    CloseParen(u8),
    Multiply,
    Divide,
    None,
    If,
    Else,
    Return(NodeID),
    Unknown(String),
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum Tag {
    Name(String),
    ID(NodeID),
}

pub struct ScriptNode {
    tokens: Vec<Token>,
    tags: Vec<Tag>,
    fallback: Option<NodeID>,
}

impl ScriptNode {
    pub fn new(script: &str) -> ScriptNode {
        let mut tokens = Vec::new();
        let mut tags = Vec::new();
        let mut words = script.split_whitespace().peekable();
        let mut open = 0;
        let mut closed = 0;

        let mut fallback = None;

        while let Some(first) = words.peek() {
            if first.starts_with("#") {
                let first = &words.next().unwrap()[1..];
                match first {
                    "name" => {
                        let next = words.next().expect("name to follow #name");
                        if next.starts_with("\"") {
                            let mut name = next[1..].to_string();
                            while !name.ends_with('\"') {
                                name.push_str(" ");
                                name.push_str(words.next().expect("name not terminated"));
                            }
                            name.pop();
                            tags.push(Tag::Name(name));
                        } else {
                            tags.push(Tag::Name(next.to_string()));
                        }
                    },
                    "id" => {
                        let next = words.next().expect("id to follow #id");
                        tags.push(Tag::ID(NodeID::from_str(next)));
                    },
                    "fallback" => {
                        let next = words.next().expect("fallback to follow #fallback");
                        fallback = Some(NodeID::from_str(next));
                    },
                    _ => bevy::log::warn!("unknown tag: #{}", first)
                }
            } else {
                break;
            }
        }

        while let Some(word) = words.next() {
            println!("parsing {}", word);
            if word.starts_with("Attribute(") {
                if word.ends_with(')') {
                    tokens.push(Token::Attribute(Attribute::from_str(&word[10..word.len() - 1])));
                } else {
                    let mut name = word[10..].to_string();
                    while !name.ends_with(')') {
                        name.push(' ');
                        name.push_str(words.next().expect(&format!("{} to have closing ')'", word)));
                    }
                    name.pop();
                    tokens.push(Token::Attribute(Attribute::new_attribute(name)));
                }
                continue;
            }
            if word.starts_with("Index(") {
                if word.ends_with(')') {
                    tokens.push(Token::Index(Attribute::from_str(word)));
                } else {
                    let mut name = word[6..].to_string();
                    while !name.ends_with(')') {
                        name.push(' ');
                        name.push_str(words.next().expect(&format!("{} to have closing ')'", word)));
                    }
                    name.pop();
                    tokens.push(Token::Attribute(Attribute::new_index(name)));
                }
                continue;
            }
            if word.starts_with("NodeID(") {
                if word.ends_with(')') {
                    tokens.push(Token::NodeID(NodeID::from_str(word)));
                } else {
                    let mut name = word[7..].to_string();
                    while !name.ends_with(')') {
                        name.push(' ');
                        name.push_str(words.next().expect(&format!("{} to have closing ')'", word)));
                    }
                    name.pop();
                    tokens.push(Token::NodeID(NodeID::from_str(&name)));
                }
                continue;
            }
            if word.starts_with(|c: char| {c.is_digit(10)}) {
                if word.starts_with("0x") {
                    tokens.push(Token::Int(usize::from_str_radix(&word[2..], 16).expect("proper hex format")));
                } if word.contains('.') {
                    let float: f32 = word.parse().expect("proper float format");
                    let float = float * 1000.;
                    tokens.push(Token::Float(float.round() as usize));
                } else {
                    tokens.push(Token::Int(word.parse().expect("proper int format")));
                }
            }
            if word.starts_with('"') {
                let mut string = String::new();
                let mut escaped = false;
                for c in word[1..].chars() {
                    if escaped {
                        string.push(c);
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        break;
                    } else {
                        string.push(c);
                    }
                }
                tokens.push(Token::String(string));
            }
            if word.starts_with('[') {
                let mut hex = Vec::with_capacity(word.len() / 2);
                let mut digits = word[1..].chars();
                while let Some(digit) = digits.next() {
                    if digit == ']' {
                        break;
                    }
                    let digit_two = digits.next().expect("hex string to have even length");
                    hex.push(u8::from_str_radix(&format!("{}{}", digit, digit_two), 16).expect("proper hex format"));
                }
                tokens.push(Token::Raw(hex));
                continue;
            }
            let token = match word {
                "==" => {Token::Equals},
                "!=" => {Token::NotEquals},
                "<=" => {Token::LestThenEq},
                ">=" => {Token::GratterThenEq},
                "<" => {Token::LessThen},
                ">" => {Token::GratterThen},
                "+" => {Token::Plus},
                "-" => {Token::Minus},
                "*" => {Token::Multiply},
                "/" => {Token::Divide},
                "(" => {open += 1; Token::OpenParen(open)},
                ")" => {closed += 1; Token::CloseParen(closed)},
                "if" => {Token::If},
                "else" => {Token::Else},
                "set" => {Token::Set},
                "none" => {Token::None},
                "return" => {Token::Return(NodeID::from_str(words.next().expect("NodeID to follow return")))},
                _ => {Token::Unknown(word.to_string())}
            };
            tokens.push(token);
        }

        ScriptNode { tokens, tags, fallback }
    }
    pub fn make_raw<T: serde::Serialize>(value: &T) -> String {
        let data = bincode::serialize(value).expect("val to serialize");
        let mut hex = String::with_capacity(data.len() * 2);
        for byte in data {
            hex.push(char::from_digit((byte >> 4) as u32, 16).expect("valid hex digit"));
            hex.push(char::from_digit((byte & 0xF) as u32, 16).expect("valid hex digit"));
        }
        hex
    }
}

fn if_condishion(state: &AnimationState, lhs: &Token, op: &Token, rhs: &Token) -> bool {
    match (lhs, rhs) {
        (Token::Index(id), Token::Int(index)) => {
            let current = state.try_get_attribute::<usize>(id);
            if current.is_none() {
                return false;
            }
            let id = current.unwrap();
            match op {
            Token::Equals => id == *index,
            Token::NotEquals => id != *index,
            Token::LestThenEq => id <= *index,
            Token::GratterThenEq => id >= *index,
            Token::LessThen => id < *index,
            Token::GratterThen => id > *index,
            _ => panic!("unsupported operator for index")
            }
        }
        (Token::Index(id), Token::Index(id2)) => {
            let id = state.try_get_attribute::<usize>(id);
            let id2 = state.try_get_attribute::<usize>(id2);
            if id.is_none() || id2.is_none() {
                return false;
            }
            match op {
            Token::Equals => id == id2,
            Token::NotEquals => id != id2,
            Token::LestThenEq => id <= id2,
            Token::GratterThenEq => id >= id2,
            Token::LessThen => id < id2,
            Token::GratterThen => id > id2,
            _ => panic!("unsupported operator for index")
            }
        },
        (Token::Index(id), Token::None) => match op {
            Token::Equals => state.try_get_attribute::<usize>(id).is_none(),
            Token::NotEquals => state.try_get_attribute::<usize>(id).is_some(),
            _ => panic!("unsupported operator for index")
        }
        (Token::Index(_), _) => {panic!("unsupported operator for index")},
        (_, _) => todo!("unsupported")
    }
}