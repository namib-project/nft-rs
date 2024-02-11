use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use strum_macros::EnumString;

use crate::types::RejectCode;
use crate::visitor::single_string_to_option_hashset_logflag;

use crate::expr::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Statements are the building blocks for rules. Each rule consists of at least one.
///
/// See <https://manpages.debian.org/testing/libnftables1/libnftables-json.5.en.html#STATEMENTS>.
pub enum Statement {
    /// `accept` verdict.
    Accept(Option<Accept>),
    /// `drop` verdict.
    Drop(Option<Drop>),
    /// `continue` verdict.
    Continue(Option<Continue>),
    /// `return` verdict.
    Return(Option<Return>),
    /// `jump` verdict. Expects a target chain name.
    Jump(JumpTarget),
    /// `goto` verdict. Expects a target chain name.
    Goto(JumpTarget),

    Match(Match),
    Counter(Option<Counter>),
    #[serde(rename = "counter")]
    /// reference to a named counter
    CounterRef(String),
    Mangle(Mangle),
    Quota(Quota),
    #[serde(rename = "quota")]
    /// reference to a named quota object
    QuotaRef(String),
    Limit(Limit),

    FWD(Option<FWD>),
    /// Disable connection tracking for the packet.
    Notrack,
    Dup(Dup),
    SNAT(Option<NAT>),
    DNAT(Option<NAT>),
    Masquerade(Option<NAT>), // masquerade is subset of NAT options
    Redirect(Option<NAT>),   // redirect is subset of NAT options
    Reject(Option<Reject>),
    Set(Set),
    Log(Option<Log>),

    #[serde(rename = "ct helper")]
    /// Enable the specified conntrack helper for this packet.
    CTHelper(String), // CT helper reference.

    Meter(Meter),
    Queue(Queue),
    #[serde(rename = "vmap")]
    VerdictMap(VerdictMap),

    #[serde(rename = "ct count")]
    CTCount(CTCount),

    #[serde(rename = "ct timeout")]
    /// Assign connection tracking timeout policy.
    CTTimeout(Expression), // CT timeout reference.

    #[serde(rename = "ct expectation")]
    /// Assign connection tracking expectation.
    CTExpectation(Expression), // CT expectation reference.

    /// This represents an xt statement from xtables compat interface.
    /// Sadly, at this point, it is not possible to provide any further information about its content.
    XT(Option<serde_json::Value>),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// `accept` verdict.
pub struct Accept {}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// `drop` verdict.
pub struct Drop {}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// `continue` verdict.
pub struct Continue {}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// `return` verdict.
pub struct Return {}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JumpTarget {
    pub target: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// This matches the expression on left hand side (typically a packet header or packet meta info) with the expression on right hand side (typically a constant value).
/// If the statement evaluates to true, the next statement in this rule is considered.
/// If not, processing continues with the next rule in the same chain.
pub struct Match {
    /// Left hand side of this match.
    pub left: Expression,
    /// Right hand side of this match.
    pub right: Expression,
    /// Operator indicating the type of comparison.
    pub op: Operator,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// This object represents a byte/packet counter.
/// In input, no properties are required.
/// If given, they act as initial values for the counter.
pub struct Counter {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Packets counted.
    pub packets: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Bytes counted.
    pub bytes: Option<usize>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// This changes the packet data or meta info.
pub struct Mangle {
    /// The packet data to be changed, given as an `exthdr`, `payload`, `meta`, `ct` or `ct helper` expression.
    pub key: Expression,
    /// Value to change data to.
    pub value: Expression,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// Creates an anonymous quota which lives in the rule it appears in.
pub struct Quota {
    /// Quota value.
    pub val: u32,
    /// Unit of `val`, e.g. `"kbytes"` or `"mbytes"`. If omitted, defaults to `"bytes"`.
    pub val_unit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Quota used so far. Optional on input. If given, serves as initial value.
    pub used: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Unit of `used`. Defaults to `"bytes"`.
    pub used_unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// If `true`, will match if quota was exceeded. Defaults to `false`.
    pub inv: Option<bool>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// Creates an anonymous limit which lives in the rule it appears in.
pub struct Limit {
    /// Rate value to limit to.
    pub rate: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Unit of `rate`, e.g. `"packets"` or `"mbytes"`. If omitted, defaults to `"packets"`.
    pub rate_unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Denominator of rate, e.g. "week" or "minutes".
    pub per: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Burst value. Defaults to `0`.
    pub burst: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Unit of `burst`, ignored if `rate_unit` is `"packets"`. Defaults to `"bytes"`.
    pub burst_unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// If `true`, will match if the limit was exceeded. Defaults to `false`.
    pub inv: Option<bool>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// Forward a packet to a different destination.
pub struct FWD {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Interface to forward the packet on.
    pub dev: Option<Expression>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Family of addr.
    pub family: Option<FWDFamily>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// IP(v6) address to forward the packet to.
    pub addr: Option<Expression>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Protocol family for `FWD`.
pub enum FWDFamily {
    IP,
    IP6,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// Duplicate a packet to a different destination.
pub struct Dup {
    /// Address to duplicate packet to.
    pub addr: Expression,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Interface to duplicate packet on. May be omitted to not specify an interface explicitly.
    pub dev: Option<Expression>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// Perform Network Address Translation.
/// Referenced by `SNAT` and `DNAT` statements.
pub struct NAT {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Address to translate to.
    pub addr: Option<Expression>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Family of addr, either ip or ip6. Required in inet table family.
    pub family: Option<NATFamily>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Port to translate to.
    pub port: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Flag(s).
    pub flags: Option<HashSet<NATFlag>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Protocol family for `NAT`.
pub enum NATFamily {
    IP,
    IP6,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
/// Flags for `NAT`.
pub enum NATFlag {
    Random,
    #[serde(rename = "fully-random")]
    FullyRandom,
    Persistent,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// Reject the packet and send the given error reply.
pub struct Reject {
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    /// Type of reject.
    pub _type: Option<RejectType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// ICMP code to reject with.
    pub expr: Option<RejectCode>,
}

impl Reject {
    pub fn new(_type: Option<RejectType>, code: Option<RejectCode>) -> Reject {
        Reject { _type, expr: code }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Types of `Reject`.
pub enum RejectType {
    #[serde(rename = "tcp reset")]
    TCPReset,
    ICMPX,
    ICMP,
    ICMPv6,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// Dynamically add/update elements to a set.
pub struct Set {
    /// Operator on set.
    pub op: SetOp,
    /// Set element to add or update.
    pub elem: Expression,
    /// Set reference.
    pub set: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Operators on `Set`.
pub enum SetOp {
    Add,
    Update,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// Log the packet.
/// All properties are optional.
pub struct Log {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Prefix for log entries.
    pub prefix: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Log group.
    pub group: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Snaplen for logging.
    pub snaplen: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "queue-threshold")]
    /// Queue threshold.
    pub queue_threshold: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Log level. Defaults to `"warn"`.
    pub level: Option<LogLevel>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "single_string_to_option_hashset_logflag"
    )]
    /// Log flags.
    pub flags: Option<HashSet<LogFlag>>,
}

impl Log {
    pub fn new(group: Option<u32>) -> Log {
        Log {
            prefix: None,
            group,
            snaplen: None,
            queue_threshold: None,
            level: None,
            flags: None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Levels of `Log`.
pub enum LogLevel {
    Emerg,
    Alert,
    Crit,
    Err,
    Warn,
    Notice,
    Info,
    Debug,
    Audit,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
/// Flags of `Log`.
pub enum LogFlag {
    #[serde(rename = "tcp sequence")]
    TCPSequence,
    #[serde(rename = "tcp options")]
    TCPOptions,
    #[serde(rename = "ip options")]
    IPOptions,
    Skuid,
    Ether,
    All,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// Apply a given statement using a meter.
pub struct Meter {
    /// Meter name.
    pub name: String,

    /// Meter key.
    pub key: Expression,

    /// Meter statement.
    pub stmt: Box<Statement>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// Queue the packet to userspace.
pub struct Queue {
    /// Queue number.
    pub num: Expression,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Queue flags.
    pub flags: Option<HashSet<QueueFlag>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
/// Flags of `Queue`.
pub enum QueueFlag {
    Bypass,
    Fanout,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename = "vmap")]
/// Apply a verdict conditionally.
pub struct VerdictMap {
    /// Map key.
    pub key: Expression,

    /// Mapping expression consisting of value/verdict pairs.
    pub data: Expression,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename = "ct count")]
/// Limit the number of connections using conntrack.
pub struct CTCount {
    /// Connection count threshold.
    pub val: Expression,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// If `true`, match if `val` was exceeded. If omitted, defaults to `false`.
    pub inv: Option<bool>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// Represents an operator for `Match`.
pub enum Operator {
    #[serde(rename = "&")]
    /// Binary AND (`&`)
    AND,

    #[serde(rename = "|")]
    /// Binary OR (`|`)
    OR,

    #[serde(rename = "^")]
    /// Binary XOR (`^`)
    XOR,

    #[serde(rename = "<<")]
    /// Left shift (`<<`)
    LSHIFT,

    #[serde(rename = ">>")]
    /// Right shift (`>>`)
    RSHIFT,

    #[serde(rename = "==")]
    /// Equal (`==`)
    EQ,

    #[serde(rename = "!=")]
    /// Not equal (`!=`)
    NEQ,

    #[serde(rename = ">")]
    /// Less than (`>`)
    LT,

    #[serde(rename = "<")]
    /// Greater than (`<`)
    GT,

    #[serde(rename = "<=")]
    /// Less than or equal to (`<=`)
    LEQ,

    #[serde(rename = ">=")]
    /// Greater than or equal to (`>=`)
    GEQ,

    #[serde(rename = "in")]
    /// Perform a lookup, i.e. test if bits on RHS are contained in LHS value (`in`)
    IN,
}
