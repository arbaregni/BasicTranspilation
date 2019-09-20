use manager::Manager;
use std::hash::Hash;
use std::collections::HashSet;
use tokenizer::Token;
use std::str::Chars;

pub enum Error {
    Zero{why: String},
    Single{why: String, at: Token},
    Many{why: String, ats: Vec<Token>},
}
impl Error {
    ///produce an error with a singular point of failure
    pub fn new(why: String, at: Token) -> Error {
        Error::Single{why, at}
    }
    ///produce an error with zero points of failure
    pub fn new_zero(why: String) -> Error { Error::Zero{why} }
    ///produce an error with many points of failure
    pub fn new_many(why: String, ats: Vec<Token>) -> Error {
        Error::Many{why, ats}
    }
    pub fn get_readout(&self, m: &Manager) -> String {
        match *self {
            Error::Zero { ref why } => why.clone(),
            Error::Single { ref why, ref at } => format!("{}\n{}", at.get_underlined(&m.source), why),
            Error::Many { ref why, ref ats } => {
                //TODO smart underlining: check to see if it all fits on one line
                let mut readout = String::new();
                for ref tok in ats {
                    readout.push_str(&tok.get_underlined(&m.source));
                    readout.push('\n');
                }
                readout.push_str(why);
                readout
            },
        }

    }
}

pub fn has_unique_elements<T>(iter: T) -> bool
    where
        T: IntoIterator,
        T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}
///replace all instances of {0}, {1}, {2}, ... with the corresponding text from `args`
/// panics on unclosed `{` or `}`, or if the text they contain is not a proper usize, or if the digit is out-of-bounds
/// `{` and `}` are escaped with single `\`
/// ```
/// let format_string = "{2} look at the {0} and {2} notice it's {1}ing";
/// let arguments = vec![String::from("world"), String::from("turn"), String::from("I")];
/// assert_eq!(vec_fmt(format_string, &arguments), String::from("I look at the world and I notice it's turning"));
/// ```
pub fn vec_fmt(format_string: &str, args: &Vec<String>) -> String  {
    let mut string = String::new();
    let mut char_iter = format_string.chars();
    while let Some(ch) = char_iter.next() {
        if ch == '{' {
            let digit = consume_until_brace(&mut char_iter).parse::<usize>().unwrap();
            string.push_str(&args[digit]);
        } else if ch == '}' {
            panic!("dangling `}`. Did you mean to escape it, like so: `\\}` ?")
        } else if ch == '\\' {
            string.push(char_iter.next().unwrap());
        } else {
            string.push(ch);
        }
    }
    string
}

fn consume_until_brace(char_iter: &mut Chars) -> String {
    let mut string = String::new();
    while let Some(ch) = char_iter.next() {
        if ch == '}' {
            return string;
        }
        string.push(ch);
    }
    panic!("encountered end of string while scanning for closing `}` (did you forget to use escape the `{` like so: `\\{` ?)")
}