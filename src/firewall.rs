//! Firewall-based quarantine enforcement via nein.
//!
//! Generates nftables rulesets for agent isolation and rate limiting
//! as quarantine enforcement actions. The generated `Firewall` can be
//! rendered and applied by the caller (e.g., daimon) using `nein::apply`.

use nein::Firewall;
use nein::chain::{Chain, ChainType, Hook, Policy};
use nein::rule::{Match, Protocol, RateUnit, Rule, Verdict};
use nein::table::{Family, Table};

/// Generate an isolation firewall that blocks all traffic for an agent.
///
/// Creates an inet table with input/output chains that drop all traffic
/// to/from the specified agent address, except established connections
/// needed for management.
#[must_use]
pub fn isolate_agent(agent_id: &str, agent_addr: &str) -> Firewall {
    let mut fw = Firewall::new();
    let table_name = format!("aegis_iso_{agent_id}");
    let mut table = Table::new(&table_name, Family::Inet);

    // Input chain: drop all traffic to the agent
    let mut input = Chain::base("input", ChainType::Filter, Hook::Input, -1, Policy::Accept);
    input.add_rule(
        Rule::new(Verdict::Drop)
            .matching(Match::DestAddr(agent_addr.to_string()))
            .comment(&format!("aegis isolate {agent_id}")),
    );

    // Output chain: drop all traffic from the agent
    let mut output = Chain::base("output", ChainType::Filter, Hook::Output, -1, Policy::Accept);
    output.add_rule(
        Rule::new(Verdict::Drop)
            .matching(Match::SourceAddr(agent_addr.to_string()))
            .comment(&format!("aegis isolate {agent_id}")),
    );

    table.add_chain(input);
    table.add_chain(output);
    fw.add_table(table);
    fw
}

/// Generate a rate-limiting firewall for an agent.
///
/// Creates rules that limit the agent's inbound and outbound traffic
/// to the specified packets-per-second rate.
#[must_use]
pub fn rate_limit_agent(agent_id: &str, agent_addr: &str, pps: u32) -> Firewall {
    let mut fw = Firewall::new();
    let table_name = format!("aegis_rl_{agent_id}");
    let mut table = Table::new(&table_name, Family::Inet);

    let mut input = Chain::base("input", ChainType::Filter, Hook::Input, -1, Policy::Accept);
    // Accept within rate limit
    input.add_rule(
        Rule::new(Verdict::Accept)
            .matching(Match::DestAddr(agent_addr.to_string()))
            .matching(Match::Limit {
                rate: pps,
                unit: RateUnit::Second,
                burst: pps * 2,
            })
            .comment(&format!("aegis rate-limit {agent_id}")),
    );
    // Drop excess
    input.add_rule(
        Rule::new(Verdict::Drop)
            .matching(Match::DestAddr(agent_addr.to_string()))
            .comment(&format!("aegis rate-limit drop {agent_id}")),
    );

    let mut output = Chain::base("output", ChainType::Filter, Hook::Output, -1, Policy::Accept);
    output.add_rule(
        Rule::new(Verdict::Accept)
            .matching(Match::SourceAddr(agent_addr.to_string()))
            .matching(Match::Limit {
                rate: pps,
                unit: RateUnit::Second,
                burst: pps * 2,
            })
            .comment(&format!("aegis rate-limit {agent_id}")),
    );
    output.add_rule(
        Rule::new(Verdict::Drop)
            .matching(Match::SourceAddr(agent_addr.to_string()))
            .comment(&format!("aegis rate-limit drop {agent_id}")),
    );

    table.add_chain(input);
    table.add_chain(output);
    fw.add_table(table);
    fw
}

/// Generate a hardened host firewall profile.
///
/// Provides a baseline security posture:
/// - Allow established/related connections
/// - Allow loopback
/// - Allow SSH (port 22)
/// - Allow ICMP echo (ping)
/// - Drop all other inbound
/// - Allow all outbound
#[must_use]
pub fn hardened_host() -> Firewall {
    let mut fw = Firewall::new();
    let mut table = Table::new("aegis_host", Family::Inet);

    let mut input = Chain::base("input", ChainType::Filter, Hook::Input, 0, Policy::Drop);
    input.add_rule(nein::rule::allow_established());
    input.add_rule(
        Rule::new(Verdict::Accept)
            .matching(Match::Iif("lo".to_string()))
            .comment("loopback"),
    );
    input.add_rule(nein::rule::allow_tcp(22).comment("SSH"));
    input.add_rule(
        Rule::new(Verdict::Accept)
            .matching(Match::Protocol(Protocol::Icmp))
            .matching(Match::IcmpType("echo-request".to_string()))
            .comment("ping"),
    );

    let output = Chain::base("output", ChainType::Filter, Hook::Output, 0, Policy::Accept);

    let forward = Chain::base("forward", ChainType::Filter, Hook::Forward, 0, Policy::Drop);

    table.add_chain(input);
    table.add_chain(output);
    table.add_chain(forward);
    fw.add_table(table);
    fw
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolate_agent_renders() {
        let fw = isolate_agent("agent-1", "10.100.0.2");
        let rendered = fw.render();
        assert!(rendered.contains("table inet aegis_iso_agent-1"));
        assert!(rendered.contains("ip daddr 10.100.0.2"));
        assert!(rendered.contains("ip saddr 10.100.0.2"));
        assert!(rendered.contains("drop"));
        assert!(rendered.contains("aegis isolate agent-1"));
    }

    #[test]
    fn isolate_agent_validates() {
        let fw = isolate_agent("agent-1", "10.100.0.2");
        assert!(fw.validate().is_ok());
    }

    #[test]
    fn rate_limit_agent_renders() {
        let fw = rate_limit_agent("agent-1", "10.100.0.2", 100);
        let rendered = fw.render();
        assert!(rendered.contains("table inet aegis_rl_agent-1"));
        assert!(rendered.contains("limit rate 100/second burst 200 packets"));
        assert!(rendered.contains("aegis rate-limit agent-1"));
        assert!(rendered.contains("aegis rate-limit drop agent-1"));
    }

    #[test]
    fn rate_limit_agent_validates() {
        let fw = rate_limit_agent("agent-1", "10.100.0.2", 50);
        assert!(fw.validate().is_ok());
    }

    #[test]
    fn hardened_host_renders() {
        let fw = hardened_host();
        let rendered = fw.render();
        assert!(rendered.contains("table inet aegis_host"));
        assert!(rendered.contains("policy drop"));
        assert!(rendered.contains("ct state { established, related }"));
        assert!(rendered.contains("dport 22"));
        assert!(rendered.contains("icmp type echo-request"));
        assert!(rendered.contains("loopback"));
    }

    #[test]
    fn hardened_host_validates() {
        let fw = hardened_host();
        assert!(fw.validate().is_ok());
    }
}
