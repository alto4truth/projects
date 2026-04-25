use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainType {
    Top,
    Bottom,
    Integer,
    Pointer,
    Boolean,
    Float,
    Interval,
    Constant,
    Struct,
    Array,
    Group,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainOperator {
    Join,
    Meet,
    Widen,
    Narrow,
    Assign,
    CompareEq,
    CompareNe,
    CompareLt,
    CompareLe,
    CompareGt,
    CompareGe,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,
    Alloc,
    Load,
    Store,
    Gep,
    Call,
    Phi,
    Select,
    Cmp,
}

#[derive(Debug, Clone)]
pub struct DomainState {
    pub is_top: bool,
    pub is_bottom: bool,
    pub domain_type: DomainType,
}

impl DomainState {
    pub fn new() -> Self {
        Self {
            is_top: true,
            is_bottom: false,
            domain_type: DomainType::Top,
        }
    }

    pub fn is_totally_unbounded(&self) -> bool {
        self.is_top && !self.is_bottom
    }

    pub fn is_conflicting(&self) -> bool {
        self.is_bottom
    }
}

impl Default for DomainState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum Domain {
    Top(TopDomain),
    Boolean(BooleanDomain),
    Integer(IntegerDomain),
    Pointer(PointerDomain),
    Interval(IntervalDomain),
    Float(FloatDomain),
    Constant(ConstantDomain),
    Group(GroupDomain),
}

impl Domain {
    pub fn domain_type(&self) -> DomainType {
        match self {
            Domain::Top(_) => DomainType::Top,
            Domain::Boolean(_) => DomainType::Boolean,
            Domain::Integer(_) => DomainType::Integer,
            Domain::Pointer(_) => DomainType::Pointer,
            Domain::Interval(_) => DomainType::Interval,
            Domain::Float(_) => DomainType::Float,
            Domain::Constant(_) => DomainType::Constant,
            Domain::Group(_) => DomainType::Group,
        }
    }

    pub fn is_top(&self) -> bool {
        match self {
            Domain::Top(d) => d.is_top,
            Domain::Boolean(d) => d.is_top,
            Domain::Integer(d) => d.is_top,
            Domain::Pointer(d) => d.is_top,
            Domain::Interval(d) => d.is_top,
            Domain::Float(d) => d.is_top,
            Domain::Constant(d) => d.is_top,
            Domain::Group(d) => d.is_top,
        }
    }

    pub fn is_bottom(&self) -> bool {
        match self {
            Domain::Top(d) => d.is_bottom,
            Domain::Boolean(d) => d.is_bottom,
            Domain::Integer(d) => d.is_bottom,
            Domain::Pointer(d) => d.is_bottom,
            Domain::Interval(d) => d.is_bottom,
            Domain::Float(d) => d.is_bottom,
            Domain::Constant(d) => d.is_bottom,
            Domain::Group(d) => d.is_bottom,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Domain::Top(d) => d.to_string(),
            Domain::Boolean(d) => d.to_string(),
            Domain::Integer(d) => d.to_string(),
            Domain::Pointer(d) => d.to_string(),
            Domain::Interval(d) => d.to_string(),
            Domain::Float(d) => d.to_string(),
            Domain::Constant(d) => d.to_string(),
            Domain::Group(d) => d.to_string(),
        }
    }

    pub fn get_state(&self) -> DomainState {
        match self {
            Domain::Top(d) => d.get_state(),
            Domain::Boolean(d) => d.get_state(),
            Domain::Integer(d) => d.get_state(),
            Domain::Pointer(d) => d.get_state(),
            Domain::Interval(d) => d.get_state(),
            Domain::Float(d) => d.get_state(),
            Domain::Constant(d) => d.get_state(),
            Domain::Group(d) => d.get_state(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TopDomain {
    is_top: bool,
    is_bottom: bool,
}

impl TopDomain {
    pub fn new() -> Self {
        Self {
            is_top: true,
            is_bottom: false,
        }
    }
}

impl Default for TopDomain {
    fn default() -> Self {
        Self::new()
    }
}

impl TopDomain {
    pub fn to_string(&self) -> String {
        if self.is_top {
            "⊤".to_string()
        } else if self.is_bottom {
            "⊥".to_string()
        } else {
            "top".to_string()
        }
    }

    pub fn get_state(&self) -> DomainState {
        DomainState {
            is_top: self.is_top,
            is_bottom: self.is_bottom,
            domain_type: DomainType::Top,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BooleanDomain {
    value: Option<bool>,
    is_top: bool,
    is_bottom: bool,
}

impl BooleanDomain {
    pub fn new() -> Self {
        Self {
            value: None,
            is_top: true,
            is_bottom: false,
        }
    }

    pub fn with_value(value: bool) -> Self {
        Self {
            value: Some(value),
            is_top: false,
            is_bottom: false,
        }
    }

    pub fn get_value(&self) -> Option<bool> {
        self.value
    }

    pub fn to_string(&self) -> String {
        if self.is_top {
            "⊤".to_string()
        } else if self.is_bottom {
            "⊥".to_string()
        } else if let Some(v) = self.value {
            v.to_string()
        } else {
            "bool".to_string()
        }
    }

    pub fn get_state(&self) -> DomainState {
        DomainState {
            is_top: self.is_top,
            is_bottom: self.is_bottom,
            domain_type: DomainType::Boolean,
        }
    }

    pub fn new_bottom() -> Self {
        Self {
            value: None,
            is_top: false,
            is_bottom: true,
        }
    }
}

impl Default for BooleanDomain {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct IntegerDomain {
    lower_bound: i64,
    upper_bound: i64,
    is_top: bool,
    is_bottom: bool,
}

impl IntegerDomain {
    pub fn new() -> Self {
        Self {
            lower_bound: i64::MIN,
            upper_bound: i64::MAX,
            is_top: true,
            is_bottom: false,
        }
    }

    pub fn with_bounds(lo: i64, hi: i64) -> Self {
        Self {
            lower_bound: lo,
            upper_bound: hi,
            is_top: false,
            is_bottom: false,
        }
    }

    pub fn singleton(value: i64) -> Self {
        Self {
            lower_bound: value,
            upper_bound: value,
            is_top: false,
            is_bottom: false,
        }
    }

    pub fn get_lower_bound(&self) -> i64 {
        self.lower_bound
    }

    pub fn get_upper_bound(&self) -> i64 {
        self.upper_bound
    }

    pub fn is_singleton(&self) -> bool {
        !self.is_top && !self.is_bottom && self.lower_bound == self.upper_bound
    }

    pub fn to_string(&self) -> String {
        if self.is_top {
            "⊤".to_string()
        } else if self.is_bottom {
            "⊥".to_string()
        } else if self.is_singleton() {
            self.lower_bound.to_string()
        } else {
            format!("[{}, {}]", self.lower_bound, self.upper_bound)
        }
    }

    pub fn get_state(&self) -> DomainState {
        DomainState {
            is_top: self.is_top,
            is_bottom: self.is_bottom,
            domain_type: DomainType::Integer,
        }
    }

    pub fn new_bottom() -> Self {
        Self {
            lower_bound: 0,
            upper_bound: 0,
            is_top: false,
            is_bottom: true,
        }
    }
}

impl Default for IntegerDomain {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PointerDomain {
    address: Option<u64>,
    is_top: bool,
    is_bottom: bool,
    is_null: bool,
}

impl PointerDomain {
    pub fn new() -> Self {
        Self {
            address: None,
            is_top: true,
            is_bottom: false,
            is_null: false,
        }
    }

    pub fn null() -> Self {
        Self {
            address: None,
            is_top: false,
            is_bottom: false,
            is_null: true,
        }
    }

    pub fn with_address(addr: u64) -> Self {
        Self {
            address: Some(addr),
            is_top: false,
            is_bottom: false,
            is_null: false,
        }
    }

    pub fn get_address(&self) -> Option<u64> {
        self.address
    }

    pub fn is_null(&self) -> bool {
        self.is_null
    }

    pub fn to_string(&self) -> String {
        if self.is_top {
            "⊤".to_string()
        } else if self.is_bottom {
            "⊥".to_string()
        } else if self.is_null {
            "null".to_string()
        } else if let Some(addr) = self.address {
            format!("ptr({})", addr)
        } else {
            "ptr".to_string()
        }
    }

    pub fn get_state(&self) -> DomainState {
        DomainState {
            is_top: self.is_top,
            is_bottom: self.is_bottom,
            domain_type: DomainType::Pointer,
        }
    }

    pub fn new_bottom() -> Self {
        Self {
            address: None,
            is_top: false,
            is_bottom: true,
            is_null: false,
        }
    }
}

impl Default for PointerDomain {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct IntervalDomain {
    lower: i64,
    upper: i64,
    is_top: bool,
    is_bottom: bool,
}

impl IntervalDomain {
    pub fn new() -> Self {
        Self {
            lower: i64::MIN,
            upper: i64::MAX,
            is_top: true,
            is_bottom: false,
        }
    }

    pub fn with_bounds(lo: i64, hi: i64) -> Self {
        Self {
            lower: lo,
            upper: hi,
            is_top: false,
            is_bottom: false,
        }
    }

    pub fn point(value: i64) -> Self {
        Self {
            lower: value,
            upper: value,
            is_top: false,
            is_bottom: false,
        }
    }

    pub fn get_lower(&self) -> i64 {
        self.lower
    }

    pub fn get_upper(&self) -> i64 {
        self.upper
    }

    pub fn is_point(&self) -> bool {
        !self.is_top && !self.is_bottom && self.lower == self.upper
    }

    pub fn contains(&self, value: i64) -> bool {
        self.is_top || (value >= self.lower && value <= self.upper)
    }

    pub fn to_string(&self) -> String {
        if self.is_top {
            "⊤".to_string()
        } else if self.is_bottom {
            "⊥".to_string()
        } else if self.is_point() {
            self.lower.to_string()
        } else {
            format!("[{}, {}]", self.lower, self.upper)
        }
    }

    pub fn get_state(&self) -> DomainState {
        DomainState {
            is_top: self.is_top,
            is_bottom: self.is_bottom,
            domain_type: DomainType::Interval,
        }
    }

    pub fn new_bottom() -> Self {
        Self {
            lower: 0,
            upper: 0,
            is_top: false,
            is_bottom: true,
        }
    }
}

impl Default for IntervalDomain {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FloatDomain {
    lower_bound: f64,
    upper_bound: f64,
    is_top: bool,
    is_bottom: bool,
}

impl FloatDomain {
    pub fn new() -> Self {
        Self {
            lower_bound: f64::NEG_INFINITY,
            upper_bound: f64::INFINITY,
            is_top: true,
            is_bottom: false,
        }
    }

    pub fn with_bounds(lo: f64, hi: f64) -> Self {
        Self {
            lower_bound: lo,
            upper_bound: hi,
            is_top: false,
            is_bottom: false,
        }
    }

    pub fn singleton(value: f64) -> Self {
        Self {
            lower_bound: value,
            upper_bound: value,
            is_top: false,
            is_bottom: false,
        }
    }

    pub fn get_lower_bound(&self) -> f64 {
        self.lower_bound
    }

    pub fn get_upper_bound(&self) -> f64 {
        self.upper_bound
    }

    pub fn to_string(&self) -> String {
        if self.is_top {
            "⊤".to_string()
        } else if self.is_bottom {
            "⊥".to_string()
        } else if (self.lower_bound - self.upper_bound).abs() < f64::EPSILON {
            self.lower_bound.to_string()
        } else {
            format!("[{}, {}]", self.lower_bound, self.upper_bound)
        }
    }

    pub fn get_state(&self) -> DomainState {
        DomainState {
            is_top: self.is_top,
            is_bottom: self.is_bottom,
            domain_type: DomainType::Float,
        }
    }

    pub fn new_bottom() -> Self {
        Self {
            lower_bound: 0.0,
            upper_bound: 0.0,
            is_top: false,
            is_bottom: true,
        }
    }
}

impl Default for FloatDomain {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum ConstantValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone)]
pub struct ConstantDomain {
    value: Option<ConstantValue>,
    is_top: bool,
    is_bottom: bool,
}

impl ConstantDomain {
    pub fn new() -> Self {
        Self {
            value: None,
            is_top: true,
            is_bottom: false,
        }
    }

    pub fn with_integer(value: i64) -> Self {
        Self {
            value: Some(ConstantValue::Integer(value)),
            is_top: false,
            is_bottom: false,
        }
    }

    pub fn with_float(value: f64) -> Self {
        Self {
            value: Some(ConstantValue::Float(value)),
            is_top: false,
            is_bottom: false,
        }
    }

    pub fn with_boolean(value: bool) -> Self {
        Self {
            value: Some(ConstantValue::Boolean(value)),
            is_top: false,
            is_bottom: false,
        }
    }

    pub fn with_string(value: String) -> Self {
        Self {
            value: Some(ConstantValue::String(value)),
            is_top: false,
            is_bottom: false,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.value, Some(ConstantValue::Integer(_)))
    }

    pub fn is_float(&self) -> bool {
        matches!(self.value, Some(ConstantValue::Float(_)))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self.value, Some(ConstantValue::Boolean(_)))
    }

    pub fn is_string(&self) -> bool {
        matches!(self.value, Some(ConstantValue::String(_)))
    }

    pub fn get_int_value(&self) -> Option<i64> {
        if let Some(ConstantValue::Integer(v)) = self.value {
            Some(v)
        } else {
            None
        }
    }

    pub fn to_string(&self) -> String {
        if self.is_top {
            "⊤".to_string()
        } else if self.is_bottom {
            "⊥".to_string()
        } else if let Some(ref v) = self.value {
            match v {
                ConstantValue::Integer(i) => i.to_string(),
                ConstantValue::Float(f) => f.to_string(),
                ConstantValue::Boolean(b) => b.to_string(),
                ConstantValue::String(s) => s.clone(),
            }
        } else {
            "constant".to_string()
        }
    }

    pub fn get_state(&self) -> DomainState {
        DomainState {
            is_top: self.is_top,
            is_bottom: self.is_bottom,
            domain_type: DomainType::Constant,
        }
    }

    pub fn new_bottom() -> Self {
        Self {
            value: None,
            is_top: false,
            is_bottom: true,
        }
    }
}

impl Default for ConstantDomain {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct GroupDomain {
    name: String,
    members: HashMap<String, Domain>,
    dependencies: HashMap<String, HashSet<String>>,
    is_top: bool,
    is_bottom: bool,
}

impl GroupDomain {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            members: HashMap::new(),
            dependencies: HashMap::new(),
            is_top: true,
            is_bottom: false,
        }
    }

    pub fn with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            members: HashMap::new(),
            dependencies: HashMap::new(),
            is_top: true,
            is_bottom: false,
        }
    }

    pub fn add_member(&mut self, key: String, domain: Domain) {
        self.members.insert(key, domain);
        self.is_top = false;
    }

    pub fn get_member(&self, key: &str) -> Option<&Domain> {
        self.members.get(key)
    }

    pub fn has_member(&self, key: &str) -> bool {
        self.members.contains_key(key)
    }

    pub fn remove_member(&mut self, key: &str) -> Option<Domain> {
        self.members.remove(key)
    }

    pub fn get_member_keys(&self) -> Vec<String> {
        self.members.keys().cloned().collect()
    }

    pub fn get_member_count(&self) -> usize {
        self.members.len()
    }

    pub fn add_dependency(&mut self, from: String, to: String) {
        self.dependencies
            .entry(from)
            .or_insert_with(HashSet::new)
            .insert(to);
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn to_string(&self) -> String {
        if self.is_top {
            "⊤".to_string()
        } else if self.is_bottom {
            "⊥".to_string()
        } else {
            format!("Group({})", self.name)
        }
    }

    pub fn get_state(&self) -> DomainState {
        DomainState {
            is_top: self.is_top,
            is_bottom: self.is_bottom,
            domain_type: DomainType::Group,
        }
    }

    pub fn new_bottom() -> Self {
        Self {
            name: String::new(),
            members: HashMap::new(),
            dependencies: HashMap::new(),
            is_top: false,
            is_bottom: true,
        }
    }
}

impl Default for GroupDomain {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DomainFactory;

impl DomainFactory {
    pub fn create_domain(domain_type: DomainType) -> Domain {
        match domain_type {
            DomainType::Top => Domain::Top(TopDomain::new()),
            DomainType::Boolean => Domain::Boolean(BooleanDomain::new()),
            DomainType::Integer => Domain::Integer(IntegerDomain::new()),
            DomainType::Pointer => Domain::Pointer(PointerDomain::new()),
            DomainType::Interval => Domain::Interval(IntervalDomain::new()),
            DomainType::Float => Domain::Float(FloatDomain::new()),
            DomainType::Constant => Domain::Constant(ConstantDomain::new()),
            DomainType::Group => Domain::Group(GroupDomain::new()),
            _ => Domain::Top(TopDomain::new()),
        }
    }

    pub fn create_top_domain(domain_type: DomainType) -> Domain {
        let mut domain = Self::create_domain(domain_type);
        domain
    }

    pub fn create_bottom_domain(domain_type: DomainType) -> Domain {
        match domain_type {
            DomainType::Boolean => Domain::Boolean(BooleanDomain::new_bottom()),
            DomainType::Integer => Domain::Integer(IntegerDomain::new_bottom()),
            DomainType::Pointer => Domain::Pointer(PointerDomain::new_bottom()),
            DomainType::Interval => Domain::Interval(IntervalDomain::new_bottom()),
            DomainType::Float => Domain::Float(FloatDomain::new_bottom()),
            DomainType::Constant => Domain::Constant(ConstantDomain::new_bottom()),
            DomainType::Group => Domain::Group(GroupDomain::new_bottom()),
            _ => Domain::Top(TopDomain::new()),
        }
    }
}
