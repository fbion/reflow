//! the routing is configured using a tree-like structure

use std::collections::BTreeMap;
use std::fmt;
use std::fmt::{Formatter, Error};

mod text;

use super::super::tcp::TcpTrafficInfo;

pub enum RoutingBranch {
    /// try them one by one, return the first match, if there is one
    /// otherwise there is no result
    Sequential(Vec<RoutingBranch>),
    Conditional(RoutingCondition),
    /// a match is found
    Final(RoutingDecision)
}


impl RoutingBranch {
    pub fn decision(&self, info: &TcpTrafficInfo)-> Option<&RoutingDecision> {
        use self::RoutingBranch::*;
        match self {
            Final(d) => Some(d),
            Conditional(c) => c.decide(info),
            Sequential(s) => {
                for r in s {
                    if let Some(d) = r.decision(info) {
                        return Some(d)
                    }
                }
                None
            }
        }
    }
}

pub enum RoutingCondition {
    Domain(BTreeMap<String, RoutingBranch>),
    IpAddr(BTreeMap<String, RoutingBranch>),
    Port(u16, Box<RoutingBranch>),
    Protocol(BTreeMap<String, RoutingBranch>),
}

impl RoutingCondition {
    fn decide(&self, info: &TcpTrafficInfo)-> Option<&RoutingDecision> {
        use self::RoutingCondition::*;
        match self {
            Domain(x) => {
                let d = info.get_domain()?;
                let d = d.as_ref();
                let r = x.get(d)?;
                r.decision(info)
            }
            _ => unimplemented!()
        }
    }
}

#[derive(Debug)]
pub struct RoutingDecision {
    route: RoutingAction,
    additional: Vec<AdditionalAction>,
}

impl RoutingDecision {
    /// simplest
    pub fn direct()-> RoutingDecision {
        RoutingDecision {
            route: RoutingAction::Direct,
            additional: vec![],
        }
    }
}


/// a chosen route
#[derive(Debug)]
enum RoutingAction {
    Direct,
    Reset,
    Named(String)
}

impl RoutingAction {
    fn named(name: &str) -> RoutingAction {
        RoutingAction::Named(name.to_string())
    }
}


#[derive(Debug)]
enum AdditionalAction {
    PrintLog,
    SaveSample,
}

impl fmt::Display for RoutingBranch {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        use self::RoutingBranch::*;
        match self {
            Sequential(x) => {
                write!(f, "[\n")?;
                for y in x {
                    write!(f, "{}\n", y)?;
                }
                write!(f, "]")?;
            }
            Conditional(x) => write!(f, "cond {}", x)?,
            Final(x) => write!(f, "{}", x)?,
        }
        Ok(())
    }
}

impl fmt::Display for RoutingCondition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        use self::RoutingCondition::*;
        match self {
            Domain(ref m) => { write!(f, "domain ")?; print_mapping(m, f)?; }
            IpAddr(ref m) => { write!(f, "ip ")?; print_mapping(m, f)?;}
            Protocol(ref m) => { write!(f, "protocol ")?; print_mapping(m, f)?;}
            Port(x, y) => { write!(f, "port eq {} => {}", x, y)?; }
        }
        Ok(())
    }
}

impl fmt::Display for RoutingDecision {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.route)?;
        for i in self.additional.iter() {
            write!(f, " and {}", i)?;
        }
        Ok(())
    }
}

impl fmt::Display for RoutingAction {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        use self::RoutingAction::*;
        match self {
            Direct => write!(f, "do direct"),
            Reset  => write!(f, "do reset"),
            Named(s) => write!(f, "use {}", s),
        }
    }
}

impl fmt::Display for AdditionalAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::AdditionalAction::*;
        match self {
            PrintLog => write!(f, "print_log"),
            SaveSample => write!(f, "save_sample"),
        }
    }
}

fn print_mapping(map: &BTreeMap<String, RoutingBranch>, f: &mut Formatter)
                 -> Result<(), Error> {
    write!(f, "{{\n")?;
    for (k,v) in map.iter() {
        write!(f, "{} => {}\n", k, v)?;
    }
    write!(f, "}}")?;
    Ok(())
}
