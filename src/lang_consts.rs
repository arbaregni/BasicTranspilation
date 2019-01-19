use type_checker::Type;
use type_checker::Type::*;

lazy_static! {

static ref NAMES: Vec<&'static str>          = vec!["equals",                "equals",           "equals",           "not-equals",           "not-equals",       "not-equals",           "lesser",           "lesser",           "lesser-equal",     "lesser-equal",         "greater",          "greater",           "greater-equal",    "greater-equal",       "add",              "add",                  "sub",              "sub",                   "mul",             "mul",              "div",              "div",             "print",        "print",        "print",        "print"];
static ref SIGNATURES: Vec<Vec<Type>>        = vec![vec![String, String],    vec![Int, Int],     vec![Boole, Boole], vec![String, String],    vec![Int, Int],     vec![Boole, Boole],    vec![Int, Int],     vec![Real, Real],    vec![Int, Int],     vec![Real, Real],       vec![Int, Int],     vec![Real, Real],   vec![Int, Int],      vec![Real, Real],       vec![Int, Int],     vec![Real, Real],       vec![Int, Int],     vec![Real, Real],       vec![Int, Int],     vec![Real, Real],  vec![Int, Int],     vec![Real, Real], vec![String],   vec![Int],      vec![Real],     vec![Boole]];
static ref RETURN_TYPES: Vec<Type>           = vec![Boole,                   Boole,              Boole,              Boole,                  Boole,               Boole,                 Boole,              Boole,               Boole,              Boole,                  Boole,              Boole,              Boole,               Boole,                  Int,               Real,                   Int,                Real,                   Int,                Real,              Real,                Real,             Void,           Void,           Void,           Void,];
static ref HANDLE_STRINGS: Vec<&'static str> = vec!["({0}={1})",             "({0}={1})",        "({0}={1})",        "({0}≠{1})",            "({0}≠{1})",         "({0}≠{1})",           "({0}<{1})",        "({0}<{1})",         "({0}≤{1})",        "({0}≤{1})",            "({0}>{1})",       "({0}>{1})",         "({0}≥{1})",         "({0}≥{1})",           "({0}+{1})",       "({0}+{1})",            "({0}-{1})",        "({0}-{1})",              "({0}*{1})",       "({0}*{1})",       "({0}/{1})",        "({0}/{1})",       "",             "",             "",             "",];
static ref CODE_STRINGS: Vec<&'static str>   = vec!["",                      "",                 "",                 "",                     "",                  "",                     "",                "",                  "",                 "",                     "",                 "",                  "",                  "",                    "",                "",                     "",                  "",                     "",                 "",                "",                 "",               "Disp {0}\n",     "Disp {0}\n",     "Disp {0}\n",     "Disp {0}\n",];
//WARN all non-empty code strings must end in a newline
}

pub fn get_id(name: &str, type_signature: &[Type]) -> Option<usize> {
    let mut iter =
        (0..NAMES.len())
        .filter(|&index| NAMES[index] == name)
        .filter(|&index| Type::check_signature(&SIGNATURES[index], type_signature))
    ;
    let elem = iter.next();
    if iter.next().is_some() {
        panic!("multiple id's found for builtin function with name: {} and signature: {:?}", name, type_signature)
    }
    elem
}

pub fn return_type_from_id(id: usize) -> Type {
    RETURN_TYPES[id].clone()
}

pub fn handle_format_string_from_id(id: usize) -> &'static str {
    HANDLE_STRINGS[id]
}

pub fn code_format_string_from_id(id: usize) -> &'static str {
    CODE_STRINGS[id]
}
