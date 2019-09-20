use tokenizer::Token;
use type_checker::Type;
use type_checker::FutureType;
use sexpr::{SexprId, Sexpr, SexprKind};
use scope::{Scope, ScopeId};
use util::Error;

use std::cell::{RefCell, Ref, RefMut};



const LABELS: [&str; 702] = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "AA", "AB", "AC", "AD", "AE", "AF", "AG", "AH", "AI", "AJ", "AK", "AL", "AM", "AN", "AO", "AP", "AQ", "AR", "AS", "AT", "AU", "AV", "AW", "AX", "AY", "AZ", "BA", "BB", "BC", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BK", "BL", "BM", "BN", "BO", "BP", "BQ", "BR", "BS", "BT", "BU", "BV", "BW", "BX", "BY", "BZ", "CA", "CB", "CC", "CD", "CE", "CF", "CG", "CH", "CI", "CJ", "CK", "CL", "CM", "CN", "CO", "CP", "CQ", "CR", "CS", "CT", "CU", "CV", "CW", "CX", "CY", "CZ", "DA", "DB", "DC", "DD", "DE", "DF", "DG", "DH", "DI", "DJ", "DK", "DL", "DM", "DN", "DO", "DP", "DQ", "DR", "DS", "DT", "DU", "DV", "DW", "DX", "DY", "DZ", "EA", "EB", "EC", "ED", "EE", "EF", "EG", "EH", "EI", "EJ", "EK", "EL", "EM", "EN", "EO", "EP", "EQ", "ER", "ES", "ET", "EU", "EV", "EW", "EX", "EY", "EZ", "FA", "FB", "FC", "FD", "FE", "FF", "FG", "FH", "FI", "FJ", "FK", "FL", "FM", "FN", "FO", "FP", "FQ", "FR", "FS", "FT", "FU", "FV", "FW", "FX", "FY", "FZ", "GA", "GB", "GC", "GD", "GE", "GF", "GG", "GH", "GI", "GJ", "GK", "GL", "GM", "GN", "GO", "GP", "GQ", "GR", "GS", "GT", "GU", "GV", "GW", "GX", "GY", "GZ", "HA", "HB", "HC", "HD", "HE", "HF", "HG", "HH", "HI", "HJ", "HK", "HL", "HM", "HN", "HO", "HP", "HQ", "HR", "HS", "HT", "HU", "HV", "HW", "HX", "HY", "HZ", "IA", "IB", "IC", "ID", "IE", "IF", "IG", "IH", "II", "IJ", "IK", "IL", "IM", "IN", "IO", "IP", "IQ", "IR", "IS", "IT", "IU", "IV", "IW", "IX", "IY", "IZ", "JA", "JB", "JC", "JD", "JE", "JF", "JG", "JH", "JI", "JJ", "JK", "JL", "JM", "JN", "JO", "JP", "JQ", "JR", "JS", "JT", "JU", "JV", "JW", "JX", "JY", "JZ", "KA", "KB", "KC", "KD", "KE", "KF", "KG", "KH", "KI", "KJ", "KK", "KL", "KM", "KN", "KO", "KP", "KQ", "KR", "KS", "KT", "KU", "KV", "KW", "KX", "KY", "KZ", "LA", "LB", "LC", "LD", "LE", "LF", "LG", "LH", "LI", "LJ", "LK", "LL", "LM", "LN", "LO", "LP", "LQ", "LR", "LS", "LT", "LU", "LV", "LW", "LX", "LY", "LZ", "MA", "MB", "MC", "MD", "ME", "MF", "MG", "MH", "MI", "MJ", "MK", "ML", "MM", "MN", "MO", "MP", "MQ", "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ", "NA", "NB", "NC", "ND", "NE", "NF", "NG", "NH", "NI", "NJ", "NK", "NL", "NM", "NN", "NO", "NP", "NQ", "NR", "NS", "NT", "NU", "NV", "NW", "NX", "NY", "NZ", "OA", "OB", "OC", "OD", "OE", "OF", "OG", "OH", "OI", "OJ", "OK", "OL", "OM", "ON", "OO", "OP", "OQ", "OR", "OS", "OT", "OU", "OV", "OW", "OX", "OY", "OZ", "PA", "PB", "PC", "PD", "PE", "PF", "PG", "PH", "PI", "PJ", "PK", "PL", "PM", "PN", "PO", "PP", "PQ", "PR", "PS", "PT", "PU", "PV", "PW", "PX", "PY", "PZ", "QA", "QB", "QC", "QD", "QE", "QF", "QG", "QH", "QI", "QJ", "QK", "QL", "QM", "QN", "QO", "QP", "QQ", "QR", "QS", "QT", "QU", "QV", "QW", "QX", "QY", "QZ", "RA", "RB", "RC", "RD", "RE", "RF", "RG", "RH", "RI", "RJ", "RK", "RL", "RM", "RN", "RO", "RP", "RQ", "RR", "RS", "RT", "RU", "RV", "RW", "RX", "RY", "RZ", "SA", "SB", "SC", "SD", "SE", "SF", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SO", "SP", "SQ", "SR", "SS", "ST", "SU", "SV", "SW", "SX", "SY", "SZ", "TA", "TB", "TC", "TD", "TE", "TF", "TG", "TH", "TI", "TJ", "TK", "TL", "TM", "TN", "TO", "TP", "TQ", "TR", "TS", "TT", "TU", "TV", "TW", "TX", "TY", "TZ", "UA", "UB", "UC", "UD", "UE", "UF", "UG", "UH", "UI", "UJ", "UK", "UL", "UM", "UN", "UO", "UP", "UQ", "UR", "US", "UT", "UU", "UV", "UW", "UX", "UY", "UZ", "VA", "VB", "VC", "VD", "VE", "VF", "VG", "VH", "VI", "VJ", "VK", "VL", "VM", "VN", "VO", "VP", "VQ", "VR", "VS", "VT", "VU", "VV", "VW", "VX", "VY", "VZ", "WA", "WB", "WC", "WD", "WE", "WF", "WG", "WH", "WI", "WJ", "WK", "WL", "WM", "WN", "WO", "WP", "WQ", "WR", "WS", "WT", "WU", "WV", "WW", "WX", "WY", "WZ", "XA", "XB", "XC", "XD", "XE", "XF", "XG", "XH", "XI", "XJ", "XK", "XL", "XM", "XN", "XO", "XP", "XQ", "XR", "XS", "XT", "XU", "XV", "XW", "XX", "XY", "XZ", "YA", "YB", "YC", "YD", "YE", "YF", "YG", "YH", "YI", "YJ", "YK", "YL", "YM", "YN", "YO", "YP", "YQ", "YR", "YS", "YT", "YU", "YV", "YW", "YX", "YY", "YZ", "ZA", "ZB", "ZC", "ZD", "ZE", "ZF", "ZG", "ZH", "ZI", "ZJ", "ZK", "ZL", "ZM", "ZN", "ZO", "ZP", "ZQ", "ZR", "ZS", "ZT", "ZU", "ZV", "ZW", "ZX", "ZY", "ZZ"];

#[derive(Debug)]
pub struct UserDefTypeManager {
    pub count: usize,
    pub name: Vec<String>,
    pub args: Vec<Vec<String>>,
    pub sgntr: Vec<Vec<FutureType>>,
}
impl UserDefTypeManager {
    pub fn new() -> UserDefTypeManager {
        UserDefTypeManager {
            count: 0,
            name: vec![],
            args: vec![],
            sgntr: vec![],
        }
    }
    /// declare a user defined type
    pub fn declare_type(&mut self, token: Token, name: String, arguments: Vec<String>, proto_signature: Vec<FutureType>) -> Result<usize, Error> {
        use util::has_unique_elements;
        if !has_unique_elements(arguments.iter()) {
            return Err(Error::new("struct contains duplicate argument names".to_string(), token));
        }
        let id = self.count;
        self.name.push(name);
        self.args.push(arguments);
        self.sgntr.push(proto_signature);
        self.count += 1;
        Ok(id)
    }
    pub fn get_field_offset(&self, struct_id: usize, field: &str) -> usize {
        self.args[struct_id]
            .iter()
            .position(|elem| elem == field)
            .expect("compiler failed to ensure that field names exist")
    }
}

#[derive(Debug)]
pub struct FuncManager {
    pub count: usize,
    pub name: Vec<String>,
    pub args: Vec<Vec<String>>,
    pub in_types: Vec<Vec<FutureType>>,
    pub out_type: Vec<FutureType>,
    pub body: Vec<SexprId>,
    pub call_site_count: usize,
}
impl FuncManager {
    pub fn new() -> FuncManager {
        FuncManager {
            count: 0,
            name: vec![],
            args: vec![],
            in_types: vec![],
            out_type: vec![],
            body: vec![],
            call_site_count: 0,
        }
    }
    /// declare a function with the following properties
    /// return its id
    pub fn declare_func(&mut self, name: String, arguments: Vec<String>, proto_signature: Vec<FutureType>, proto_out_type: FutureType, body: SexprId) -> Result<usize, Error> {
        let id = self.count;
        self.name.push(name);
        self.args.push(arguments);
        self.in_types.push(proto_signature);
        self.out_type.push(proto_out_type);
        self.body.push(body);
        //TODO give all functions their labels later
        //self.func_labels.push(lbl);
        self.count += 1;
        Ok(id)
    }
    /// get the id for the function body
    pub fn func_body(&self, id: usize) -> SexprId {
        self.body[id]
    }

    /*/// call a closure on a mutable reference to the function's body
    pub fn mutate_func_body<T>(&mut self, id: usize, mut closure: impl FnMut(&mut Sexpr, &mut Manager) -> T) -> T {
        let mut x = self.func_body[id].take().expect("func body was removed from option -- found None variant");
        let res = closure(&mut x, self);
        self.func_body[id] = Some(x);
        res
    }*/
}

#[derive(Debug)]
pub struct BuiltinManager {
    pub name: Vec<String>,
    pub in_types: Vec<Vec<Type>>,
    pub out_type: Vec<Type>,
    pub handle: Vec<String>,
    pub code: Vec<String>,
}
impl BuiltinManager {
    pub fn new() -> BuiltinManager {
        let mut s = BuiltinManager {
            name: vec![],
            in_types: vec![],
            out_type: vec![],
            handle: vec![],
            code: vec![],
        };
        let header = include_str!("builtin_header");
        let mut iter = header.lines();
        while let Some(line) = iter.next() {
            if line.chars().all(char::is_whitespace) { continue; }

            let words = line.split_whitespace().collect::<Vec<&str>>();

            let (ident, name) = (words[0], words[1]);
            match ident {
                "func" => {
                    let typeline = iter.next().expect("missing typeline").split(" -> ").collect::<Vec<&str>>();
                    let signature = typeline[0].split_whitespace().map(|w| Type::from_text_primitive(w).expect("not a builtin type")).collect::<Vec<Type>>();
                    let out_type = Type::from_text_primitive(typeline[1]).expect("not a builtin type");
                    let handle = iter.next().expect("no handle template found");
                    let code = iter.next().expect("no code template found");
                    let _sep = iter.next().expect("no separator found");

                    s.name.push(name.to_string());
                    s.in_types.push(signature);
                    s.out_type.push(out_type);
                    s.handle.push(handle.to_string());
                    s.code.push(
                        if code.len() == 0 { String::new() }
                            else { format!("{}\n", code) }
                    );
                }

                ident => panic!("mangled builtin hint: {}", ident)
            }
        }
        s
    }
    pub fn resolve_name(&self, name: &str, signature: &[Type]) -> Option<usize> {
        let mut iter =
            (0..self.name.len())
                .filter(|&index| self.name[index] == name)
                .filter(|&index| Type::check_signature(&self.in_types[index], signature))
        ;
        let elem = iter.next();
        if iter.next().is_some() {
            panic!("multiple id's found for builtin function with name: {} and signature: {:?}", name, signature)
        }
        elem
    }
}

#[derive(Debug)]
pub struct Manager {
    pub udt_manager: UserDefTypeManager,
    pub func_manager: FuncManager,
    pub builtin_manager: BuiltinManager,
    pub source: String,
    pub all_sexprs: Vec<RefCell<Sexpr>>,
    pub sexpr_result_types: Vec<Type>,
    pub sexpr_scopes: Vec<ScopeId>,
    pub top_level_sexprs: Vec<SexprId>,
    pub all_scopes: Vec<Scope>,
}
impl Manager {
    pub fn new(source: String) -> Manager {
        Manager {
            udt_manager: UserDefTypeManager::new(),
            func_manager: FuncManager::new(),
            builtin_manager: BuiltinManager::new(),
            source,
            all_sexprs: vec![],
            sexpr_result_types: vec![],
            sexpr_scopes: vec![],
            top_level_sexprs: vec![],
            all_scopes: vec![],

        }
    }
    pub fn lookup_user_def_type(name: &str, type_names: &Vec<String>) -> Option<Type> {
        type_names
            .iter()
            .position(|ref type_name| **type_name == name)
            .map(|index| Type::CustomType(name.to_string(), index))
    }
    /// upgrade all FutureTypes into a concrete type
    pub fn initialize_type_info(&mut self) -> Result<(), Error> {
        fn upgrade_all<'a, 'b>(types_to_upgrade: impl Iterator<Item = &'a mut FutureType>, source: &str, names_of_types: &'b Vec<String>) -> Result<(), Error> {
            types_to_upgrade
                .map(|elem: &mut FutureType| elem.upgrade(source, names_of_types))
                .collect::<Result<Vec<()>, Error>>()?;
            Ok(())
        }
        let types_to_upgrade =
            self.udt_manager.sgntr
                .iter_mut()
                .flatten()
                .chain(
                    self.func_manager.in_types
                        .iter_mut()
                        .flatten()
                )
                .chain(
                    self.func_manager.out_type
                        .iter_mut()
                );
        upgrade_all(types_to_upgrade, &self.source, &self.udt_manager.name)
    }
    /// declare a call site that calls the function with the given id
    /// return the call site id (to be used by the functions to return)
    pub fn declare_call_site(&mut self, _id: usize) -> Result<usize, Error> {
        //TODO id can be used in optimizing
        let call_site_id = self.func_manager.call_site_count;
        self.func_manager.call_site_count += 1;
        Ok(call_site_id)
    }

    /// borrow the corresponding s-expr
    pub fn sexpr(&self, sexpr_id: SexprId) -> Ref<Sexpr> { self.all_sexprs[sexpr_id.index].borrow() }
    /// mutable borrow the corresponding s-expr
    pub fn sexpr_mut(&self, sexpr_id: SexprId) -> RefMut<Sexpr> { self.all_sexprs[sexpr_id.index].borrow_mut() }
    /// get the text of the head from this s-expr's token
    pub fn text(&self, sexpr_id: SexprId) -> &str { self.sexpr(sexpr_id).token.get_text(&self.source) }
    /// get the kind of this sexpr
    pub fn kind(&self, sexpr_id: SexprId) -> &SexprKind {
        use std::ops::Deref;
        &Box::deref(&self.sexpr(sexpr_id).kind)
    }
    /// get the id of this s-expr's scope
    pub fn scope_of(&self, sexpr_id: SexprId) -> ScopeId { self.sexpr_scopes[sexpr_id.index] }
    ///returns a copy of its identifier if it is a valid one, otherwise returns an error
    pub fn get_ident(&self, sexpr_id: SexprId) -> Result<String, Error> {
        match *self.sexpr(sexpr_id).kind {
            SexprKind::Identifier => Ok(self.text(sexpr_id).to_string()),
            _ => Err(Error::new(format!("not a valid identifier"), self.sexpr(sexpr_id).token))
        }
    }

    /// get the scope object associated with this id
    pub fn scope(&self, scope_id: ScopeId) -> &Scope { &self.all_scopes[scope_id.index] }
    pub fn scope_mut(&mut self, scope_id: ScopeId) -> &mut Scope { &mut self.all_scopes[scope_id.index] }
    /// add a child to this scope, returning it's id
    pub fn create_child(&mut self, scope_id: ScopeId) -> ScopeId {
        let child = Scope::new_to_vec(&mut self.all_scopes,Some(scope_id), false);
        self.scope_mut(scope_id).children.push(child);
        child
    }

    pub fn bind_variable(&mut self, scope_id: ScopeId, name: String, opt_type: Option<&Type>) {
        let bound_count = self.scope(scope_id).variable_types.len();
        self.scope_mut(scope_id).declared_variables.insert(name, bound_count);
        self.scope_mut(scope_id).variable_types.push(opt_type.map(|t: &Type| t.clone()));
    }
    /// resolve a variable name into the scope id and variable index
    pub fn resolve_variable(&self, scope_id: ScopeId, name: &str) -> Option<(ScopeId, usize)> {
        // if we can't get the name, check if we have a parent. if we do, then ask them
        if let Some(&var_id) = self.scope(scope_id).declared_variables.get(name) {
            Some((scope_id, var_id))
        } else {
            self.scope(scope_id).parent.and_then(|parent| self.resolve_variable(parent, name))
        }
    }

    pub fn inform_var_type(&mut self, scope_id: ScopeId, name: &str, var_type: &Type) {
        let (scope_id, var_id) = self.resolve_variable(scope_id, name).expect("could not find variable when creating types");
        if self.scope(scope_id).variable_types[var_id].is_some() {
            panic!("conflicting types for variable with name: {}, trying to set to type: {}, found: {:?}, and in {:?}",
                   name,
                   var_type,
                   self.scope(scope_id).variable_types[var_id].as_ref(),
                   scope_id)
        }
        self.scope_mut(scope_id).variable_types[var_id] = Some(var_type.clone());
    }

    pub fn bind_func(&mut self, scope_id: ScopeId, func_id: usize) { self.scope_mut(scope_id).declared_functions.push(func_id); }
    pub fn resolve_func(&self, scope_id: ScopeId, name: &str, signature: &[Type]) -> Option<usize> {
        for &id in self.scope(scope_id).declared_functions.iter() {
            if *self.func_manager.name[id] == *name && Type::check_futures_signature(&self.func_manager.in_types[id], signature) {
                return Some(id);
            }
        }
        self.scope(scope_id).parent.and_then(|parent| self.resolve_func(parent, name, signature))
    }

    pub fn bind_struct_init(&mut self, scope_id: ScopeId, struct_id: usize) { self.scope_mut(scope_id).declared_structs.push(struct_id); }
    pub fn resolve_struct_init(&self, scope_id: ScopeId, name: &str, signature: &[Type]) -> Option<usize> {
        for &id in self.scope(scope_id).declared_structs.iter() {
            if *self.udt_manager.name[id] == *name && Type::check_futures_signature(&self.func_manager.in_types[id], signature) {
                return Some(id);
            }
        }
        self.scope(scope_id).parent.and_then(|parent| self.resolve_struct_init(parent, name, signature))
    }
}