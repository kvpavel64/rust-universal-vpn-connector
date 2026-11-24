/// Xray routing configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootRouting {
    /// Domain resolution strategy.
    #[serde(rename = "domainStrategy")]
    pub domain_strategy: String,
    /// Domain matcher implementation.
    #[serde(rename = "domainMatcher")]
    pub domain_matcher: String,
    /// Ordered routing rules.
    pub rules: Vec<RootRoutingRulesItem>,
    /// Optional outbound balancers.
    pub balancers: Option<Vec<RootRoutingBalancersItem>>,
}

/// Routing rule.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootRoutingRulesItem {
    /// Rule type.
    pub r#type: String,
    /// Optional network filter.
    pub network: Option<String>,
    /// Optional port filter.
    pub port: Option<String>,
    /// Target outbound tag.
    #[serde(rename = "outboundTag")]
    pub outbound_tag: Option<String>,
    /// Domain match expressions.
    pub domain: Option<Vec<String>>,
    /// Protocol match expressions.
    pub protocol: Option<Vec<String>>,
    /// Target balancer tag.
    #[serde(rename = "balancerTag")]
    pub balancer_tag: Option<String>,
}

/// Outbound balancer definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootRoutingBalancersItem {
    /// Balancer tag.
    pub tag: String,
    /// Outbound selector expressions.
    pub selector: Vec<String>,
    /// Balancer strategy.
    pub strategy: RootRoutingBalancersItemStrategy,
    /// Fallback outbound tag.
    #[serde(rename = "fallbackTag")]
    pub fallback_tag: String,
}

/// Balancer strategy definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootRoutingBalancersItemStrategy {
    /// Strategy type.
    pub r#type: String,
    /// Strategy-specific settings.
    pub settings: RootRoutingBalancersItemStrategySettings,
}

/// Balancer strategy settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootRoutingBalancersItemStrategySettings {
    /// Maximum round-trip time.
    #[serde(rename = "maxRTT")]
    pub max_rtt: String,
    /// Expected baseline value.
    pub expected: i64,
    /// Baseline values.
    pub baselines: Vec<String>,
    /// Allowed tolerance.
    pub tolerance: i64,
    /// Cost rules.
    pub costs: Vec<RootRoutingBalancersItemStrategySettingsCostsItem>,
}

/// One balancer cost rule.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootRoutingBalancersItemStrategySettingsCostsItem {
    /// Whether the match expression is a regular expression.
    pub regexp: Option<bool>,
    /// Match expression.
    pub r#match: String,
    /// Cost value.
    pub value: i64,
}
