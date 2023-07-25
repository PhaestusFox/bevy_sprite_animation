use crate::prelude::RunError;
use crate::serde::{LoadNode, ReflectLoadNode};
use crate::{
    prelude::{AnimationNodeTrait, Attribute, BevySpriteAnimationError, NodeId, NodeResult},
    state::AnimationState,
};
use bevy::log::error;
use bevy::{prelude::AssetServer, reflect::Reflect};
use std::str::FromStr;

impl AnimationNodeTrait for ScriptNode {
    fn run(&self, state: &mut crate::state::AnimationState) -> Result<NodeResult, RunError> {
        let mut index = 0;
        while index < self.tokens.len() {
            match &self.tokens[index] {
                Token::If => {
                    if if_condishion(
                        state,
                        &self.tokens[index + 1],
                        &self.tokens[index + 2],
                        &self.tokens[index + 3],
                    ) {
                        index += 4;
                    } else {
                        index += 7;
                    }
                }
                Token::Set => {
                    let key = match &self.tokens[index + 1] {
                        Token::Attribute(i) => i,
                        _ => {
                            panic!("unimplemented `set {:?}`", self.tokens[index + 1])
                        }
                    };
                    match &self.tokens[index + 2] {
                        Token::Raw(_) => {
                            todo!();
                            // let data = state.get_attribute_raw_mut(&key);
                            //*data = v.clone();
                        }
                        Token::Ron(data) => {
                            #[cfg(feature = "ron")]
                            {
                                if let Err(e) = state.set_from_ron(&key, data) {
                                    return Err(match e {
                                        crate::error::StateError::NotFound => RunError::Custom(
                                            format!("ScriptNode: {} is not set;\n
                                            currently it needs to know the type it is turning the string into :|\n
                                            feel free to do a pull request if you have a better way", key)
                                        ),
                                        crate::error::StateError::WrongType => RunError::Custom(
                                            format!("ScriptNode: {} has the wrong type?\n
                                            this is a bug, the type shoud be decided by the one already there", key)
                                        ),
                                        crate::error::StateError::SetByRon(e) => RunError::Custom(
                                            format!("ScriptNode: {} Ron Failed?\n
                                            {:?}", key, e)
                                        ),
                                        crate::error::StateError::NotRegistered(e) => RunError::Custom(
                                            format!("ScriptNode: {} type not reflect\n
                                            {:?}", key, e)
                                        ),
                                    });
                                }
                            }
                            #[cfg(not(feature = "ron"))]
                            {
                                return Err(NodeResult::Error(format!(
                                    "tried to set {:?} = Ron({}) without ron feature",
                                    key, data
                                )));
                            }
                        }
                        Token::Int(val) => {
                            if key.is_index() {
                                state.set_attribute(key.clone(), *val);
                            } else {
                                panic!("unimplemented only Index(_) can be set to int")
                            }
                        }
                        _ => panic!(
                            "unimplemented `set {:?} = {:?}`",
                            key,
                            self.tokens[index + 2]
                        ),
                    }
                    index += 3;
                }
                Token::Return(id) => {
                    return Ok(NodeResult::Next(id.to_static()));
                }
                _ => {
                    bevy::log::info!("pointer {}", index);
                    bevy::log::info!("statck = {:?}", self.tokens);
                    todo!("Token {:?}", self.tokens[index])
                }
            }
        }
        if let Some(fallback) = &self.fallback {
            bevy::log::warn!("fallback {:?} used", fallback);
            Ok(NodeResult::Next(fallback.to_static()))
        } else {
            Err(RunError::Custom(
                "ScriptNode: failed to find a node to return and no fallback was set;\n
            use #fallback followed by a NodeId at the begging of you script to set a fallback node"
                    .to_string(),
            ))
        }
    }

    fn name(&self) -> &str {
        for tag in self.tags.iter() {
            if let Tag::Name(name) = tag {
                return &name;
            }
        }
        "unnamed stript; add #name to the first line to add a name"
    }

    fn id(&self) -> NodeId {
        let mut has_name = None;
        for tag in self.tags.iter() {
            if let Tag::Name(name) = tag {
                has_name = Some(name);
            }
            if let Tag::ID(id) = tag {
                return id.to_static();
            }
        }
        if let Some(name) = has_name {
            NodeId::from_name(name)
        } else {
            panic!()
        }
    }

    fn set_id(&mut self, new: NodeId<'_>) {
        for tag in self.tags.iter_mut() {
            if let Tag::ID(id) = tag {
                *id = new.to_static();
                return;
            }
        }
        self.tags.push(Tag::ID(new.to_static()))
    }

    fn serialize(
        &self,
        data: &mut String,
        _: &AssetServer,
    ) -> Result<(), BevySpriteAnimationError> {
        data.push_str("ScriptNode(");
        data.push(' ');
        for tag in self.tags.iter() {
            data.push_str(&tag.to_string());
            data.push('\n');
        }
        if let Some(fallback) = &self.fallback {
            data.push_str("#fallback ");
            data.push_str(&format!("{}", fallback));
            data.push('\n');
        }
        for token in self.tokens.iter() {
            data.push_str(&token.to_string());
            data.push(' ');
        }
        data.push_str("),\n\t");
        Ok(())
    }

    #[cfg(feature = "dot")]
    fn dot(&self, this: NodeId<'_>, out: &mut String, _: &AssetServer) {
        this.dot(out);
        out.push_str(&format!(" [label=\"{}\"];\n", self.name()));
        if let Some(fallback) = &self.fallback {
            this.dot(out);
            out.push_str(" -> ");
            fallback.dot(out);
            out.push_str(&format!("[label=\"Fallback\", color=red];\n"));
        }
        for token in self.tokens.iter() {
            if let Token::Return(id) = token {
                this.dot(out);
                out.push_str(" -> ");
                id.dot(out);
                out.push_str(&format!(";\n"));
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
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
    NodeId(NodeId<'static>),
    OpenParen(u8),
    CloseParen(u8),
    Multiply,
    Divide,
    None,
    If,
    Else,
    Return(NodeId<'static>),
    Ron(String),
    Unknown(String),
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Token::Return(id) => format!("return {}", id),
            Token::Int(i) => format!("{}", i),
            Token::Equals => "==".to_string(),
            Token::NotEquals => "!=".to_string(),
            Token::LessThen => "<".to_string(),
            Token::LestThenEq => "<=".to_string(),
            Token::GratterThen => ">".to_string(),
            Token::GratterThenEq => ">=".to_string(),
            Token::Plus => "+".to_string(),
            Token::Minus => "-".to_string(),
            Token::String(s) => format!("\"{}\"", s),
            Token::Raw(v) => {
                let mut res = String::with_capacity(v.len() * 2 + 2);
                res.push('[');
                for b in v {
                    res.push_str(&format!("{:#04X}", b)[2..]);
                }
                res.push(']');
                res
            }
            Token::If => "if".to_string(),
            Token::Else => "else".to_string(),
            Token::Set => "set".to_string(),
            Token::Attribute(att) => format!("{}", att),
            Token::Ron(data) => format!("Ron({})", data),
            _ => panic!("unimplemented `to_string` for {:?}", self),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Tag {
    Name(String),
    ID(NodeId<'static>),
}

impl ToString for Tag {
    fn to_string(&self) -> String {
        match self {
            Tag::Name(name) => format!("#name \"{}\"", name),
            Tag::ID(id) => format!("#id {}", id),
        }
    }
}

#[derive(Reflect)]
#[reflect(LoadNode)]
pub struct ScriptNode {
    #[reflect(ignore)]
    tokens: Vec<Token>,
    #[reflect(ignore)]
    tags: Vec<Tag>,
    #[reflect(ignore)]
    fallback: Option<NodeId<'static>>,
}

impl ScriptNode {
    pub fn new(script: &str) -> ScriptNode {
        let mut tokens = Vec::new();
        let mut tags = Vec::new();
        let script = script.trim();
        let script = if script.starts_with('(') {
            &script[1..script.len() - 1]
        } else {
            script
        };
        let mut words = script.split_whitespace().peekable();
        let mut open = 0;
        let mut closed = 0;

        let mut fallback = None;

        while let Some(first) = words.peek() {
            if first.trim().starts_with("#") {
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
                    }
                    "id" => {
                        let next = words.next().expect("id to follow #id");
                        tags.push(Tag::ID(NodeId::from_str(next).unwrap()));
                    }
                    "fallback" => {
                        let next = words.next().expect("fallback to follow #fallback");
                        fallback = Some(NodeId::from_str(next).unwrap());
                    }
                    _ => bevy::log::warn!("unknown tag: #{}", first),
                }
            } else {
                break;
            }
        }

        while let Some(word) = words.next() {
            if let Some(word) = word.strip_prefix("Attribute") {
                let named = word.starts_with("(\"");
                tokens.push(Token::Attribute(if word.ends_with(')') {
                    if named {
                        Attribute::new_attribute(word[2..word.len() - 2].to_string())
                    } else {
                        Attribute::CustomId(
                            word[1..word.len() - 1]
                                .parse::<u64>()
                                .expect("Attribute(_) must be a number"),
                        )
                    }
                } else {
                    let mut name = word[2..].to_string();
                    while !name.ends_with(')') {
                        name.push(' ');
                        name.push_str(
                            words
                                .next()
                                .expect(&format!("{} to have closing ')'", word)),
                        );
                    }
                    name.pop();
                    name.pop();
                    Attribute::new_attribute(name)
                }));
                continue;
            }
            if let Some(word) = word.strip_prefix("Index") {
                let named = word.starts_with("(\"");
                tokens.push(Token::Attribute(if word.ends_with(')') {
                    if named {
                        Attribute::new_index(word[2..word.len() - 2].to_string())
                    } else {
                        Attribute::IndexId(
                            word[1..word.len() - 1]
                                .parse::<u64>()
                                .expect("Index(_) must be a number"),
                        )
                    }
                } else {
                    let mut name = word[2..].to_string();
                    while !name.ends_with(')') {
                        name.push(' ');
                        name.push_str(
                            words
                                .next()
                                .expect(&format!("{} to have closing ')'", word)),
                        );
                    }
                    name.pop();
                    name.pop();
                    Attribute::new_index(name)
                }));
                continue;
            }
            if word.starts_with("NodeId(") {
                if word.ends_with(')') {
                    tokens.push(Token::NodeId(NodeId::from_str(word).unwrap()));
                } else {
                    let mut name = word[7..].to_string();
                    while !name.ends_with(')') {
                        name.push(' ');
                        name.push_str(
                            words
                                .next()
                                .expect(&format!("{} to have closing ')'", word)),
                        );
                    }
                    name.pop();
                    tokens.push(Token::NodeId(NodeId::from_str(&name).unwrap()));
                }
                continue;
            }
            if word.starts_with("NodeName(") {
                if word.ends_with(')') {
                    tokens.push(Token::NodeId(NodeId::from_str(word).unwrap()));
                } else {
                    let mut name = word[9..].to_string();
                    while !name.ends_with(')') {
                        name.push(' ');
                        name.push_str(
                            words
                                .next()
                                .expect(&format!("{} to have closing ')'", word)),
                        );
                    }
                    name.pop();
                    tokens.push(Token::NodeId(NodeId::from_str(&name).unwrap()));
                }
                continue;
            }
            if word.starts_with(|c: char| c.is_digit(10)) {
                if word.starts_with("0x") {
                    tokens.push(Token::Int(
                        usize::from_str_radix(&word[2..], 16).expect("proper hex format"),
                    ));
                }
                if word.contains('.') {
                    let float: f32 = word.parse().expect("proper float format");
                    let float = float * 1000.;
                    tokens.push(Token::Float(float.round() as usize));
                } else {
                    tokens.push(Token::Int(word.parse().expect("proper int format")));
                }
                continue;
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
                    hex.push(
                        u8::from_str_radix(&format!("{}{}", digit, digit_two), 16)
                            .expect("proper hex format"),
                    );
                }
                tokens.push(Token::Raw(hex));
                continue;
            }
            if word.starts_with("Ron(") {
                let mut ron = String::from(&word[5..]);
                while ron.matches('(').collect::<Vec<&str>>().len()
                    > ron.matches(')').collect::<Vec<&str>>().len()
                {
                    ron.push_str(words.next().expect("there to be another word"));
                }
                if !ron.ends_with(')') {
                    error!("ron(_) should not have trailing chars");
                    continue;
                }
                #[cfg(not(feature = "ron"))]
                {
                    warn!("using ron(_) without ron feature enabled\n");
                }
                ron.pop();
                ron.pop();
                tokens.push(Token::Ron(ron));
                continue;
            }
            let token = match word {
                "==" => Token::Equals,
                "!=" => Token::NotEquals,
                "<=" => Token::LestThenEq,
                ">=" => Token::GratterThenEq,
                "<" => Token::LessThen,
                ">" => Token::GratterThen,
                "+" => Token::Plus,
                "-" => Token::Minus,
                "*" => Token::Multiply,
                "/" => Token::Divide,
                "(" => {
                    open += 1;
                    Token::OpenParen(open)
                }
                ")" => {
                    closed += 1;
                    Token::CloseParen(closed)
                }
                "if" => Token::If,
                "else" => Token::Else,
                "set" => Token::Set,
                "none" => Token::None,
                "return" => Token::Return(
                    NodeId::from_str(words.next().expect("NodeId to follow return")).unwrap(),
                ),
                _ => Token::Unknown(word.to_string()),
            };
            tokens.push(token);
        }

        ScriptNode {
            tokens,
            tags,
            fallback,
        }
    }
}

fn if_condishion(state: &AnimationState, lhs: &Token, op: &Token, rhs: &Token) -> bool {
    match (lhs, rhs) {
        (Token::Attribute(id), Token::Int(index)) => {
            let Ok(current) = state.get_attribute::<usize>(id) else {return false;};
            match op {
                Token::Equals => current == index,
                Token::NotEquals => current != index,
                Token::LestThenEq => current <= index,
                Token::GratterThenEq => current >= index,
                Token::LessThen => current < index,
                Token::GratterThen => current > index,
                _ => panic!("unsupported operator for index"),
            }
        }
        (Token::Attribute(id), Token::Attribute(id2)) => {
            let Ok(id) = state.get_attribute::<usize>(id) else {return false;};
            let Ok(id2) = state.get_attribute::<usize>(id2) else {return false;};
            match op {
                Token::Equals => id == id2,
                Token::NotEquals => id != id2,
                Token::LestThenEq => id <= id2,
                Token::GratterThenEq => id >= id2,
                Token::LessThen => id < id2,
                Token::GratterThen => id > id2,
                _ => panic!("unsupported operator for index"),
            }
        }
        (Token::Attribute(id), Token::None) => match op {
            Token::Equals => state.get_attribute::<usize>(id).is_err(),
            Token::NotEquals => state.get_attribute::<usize>(id).is_ok(),
            _ => panic!("unsupported operator for index"),
        },
        (Token::Attribute(_), _) => {
            panic!("unsupported operator for index")
        }
        (_, _) => todo!("unsupported"),
    }
}

impl LoadNode for ScriptNode {
    fn load<'b>(
        s: &str,
        _: &mut bevy::asset::LoadContext<'b>,
        _dependencies: &mut Vec<bevy::asset::AssetPath<'static>>,
    ) -> Result<crate::AnimationNode, crate::error::LoadError> {
        Ok(crate::AnimationNode::new(ScriptNode::new(s)))
    }
}
