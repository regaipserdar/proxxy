use async_graphql::{Context, Object, Schema, Subscription, SimpleObject};
use tokio_stream::Stream;
use crate::pb::{TrafficEvent, traffic_event, SystemMetricsEvent};
use crate::Database;
use std::sync::Arc;
use tokio_stream::StreamExt;

pub type ProxySchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> &str {
        "Hello from Proxxy!"
    }

    async fn requests(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<TrafficEventGql>> {
         let db = ctx.data::<Arc<Database>>()?;
         let events = db.get_recent_requests(50).await.map_err(|e| async_graphql::Error::new(e.to_string()))?;
         
         Ok(events.into_iter().map(TrafficEventGql::from).collect())
    }

    async fn agents(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<AgentGql>> {
        let registry = ctx.data::<Arc<crate::AgentRegistry>>()?;
        let agents = registry.list_agents();
        Ok(agents.into_iter().map(|a| AgentGql {
            id: a.id,
            hostname: a.hostname,
        }).collect())
    }

    async fn system_metrics(&self, ctx: &Context<'_>, agent_id: Option<String>, limit: Option<i32>) -> async_graphql::Result<Vec<SystemMetricsGql>> {
        let db = ctx.data::<Arc<Database>>()?;
        let limit = limit.unwrap_or(60) as i64;
        let events = db.get_recent_system_metrics(agent_id.as_deref(), limit).await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        
        Ok(events.into_iter().map(SystemMetricsGql::from).collect())
    }

    async fn current_system_metrics(&self, ctx: &Context<'_>, agent_id: String) -> async_graphql::Result<Option<SystemMetricsGql>> {
        let db = ctx.data::<Arc<Database>>()?;
        let events = db.get_recent_system_metrics(Some(&agent_id), 1).await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        
        Ok(events.into_iter().next().map(SystemMetricsGql::from))
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn intercept(&self, _id: String, _action: String) -> bool {
        // TODO: Implement interception logic
        true
    }
}

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn events(&self, ctx: &Context<'_>) -> impl Stream<Item = TrafficEventGql> {
        let broadcast = ctx.data::<tokio::sync::broadcast::Sender<TrafficEvent>>().expect("Broadcast missing").clone();
        let rx = broadcast.subscribe();
        
        tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|res| 
            res.ok().map(|e| TrafficEventGql::from(e))
        )
    }

    async fn system_metrics_updates(&self, ctx: &Context<'_>, agent_id: Option<String>) -> impl Stream<Item = SystemMetricsGql> {
        let broadcast = ctx.data::<tokio::sync::broadcast::Sender<SystemMetricsEvent>>().expect("Metrics broadcast missing").clone();
        let rx = broadcast.subscribe();
        
        tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(move |res| {
            let agent_id = agent_id.clone();
            res.ok().and_then(|e| {
                // Filter by agent_id if specified
                if let Some(ref filter_id) = agent_id {
                    if e.agent_id != *filter_id {
                        return None;
                    }
                }
                Some(SystemMetricsGql::from(e))
            })
        })
    }
}

// -- GraphQL Types Wrapper --
// Since Proto types are generated, we might need to wrap them or impl scalar
// Simpler: Define GQL mirrors

#[derive(SimpleObject)]
pub struct TrafficEventGql {
    pub request_id: String,
    pub method: Option<String>,
    pub url: Option<String>,
    pub status: Option<i32>,
}

impl From<TrafficEvent> for TrafficEventGql {
    fn from(e: TrafficEvent) -> Self {
        let mut method = None;
        let mut url = None;
        let mut status = None;
        
        match e.event {
             Some(traffic_event::Event::Request(req)) => {
                 method = Some(req.method);
                 url = Some(req.url);
             },
             Some(traffic_event::Event::Response(res)) => {
                 status = Some(res.status_code);
             },
             _ => {}
        }

        Self {
            request_id: e.request_id,
            method,
            url,
            status,
        }
    }
}

#[derive(SimpleObject)]
pub struct AgentGql {
    pub id: String,
    pub hostname: String,
}

#[derive(SimpleObject)]
pub struct SystemMetricsGql {
    pub agent_id: String,
    pub timestamp: i64,
    pub cpu_usage_percent: f32,
    pub memory_used_bytes: String,
    pub memory_total_bytes: String,
    pub network_rx_bytes_per_sec: String,
    pub network_tx_bytes_per_sec: String,
    pub disk_read_bytes_per_sec: String,
    pub disk_write_bytes_per_sec: String,
    pub process_cpu_percent: f32,
    pub process_memory_bytes: String,
    pub process_uptime_seconds: i32,
}

impl From<SystemMetricsEvent> for SystemMetricsGql {
    fn from(event: SystemMetricsEvent) -> Self {
        let metrics = event.metrics.unwrap_or_default();
        let network = metrics.network.unwrap_or_default();
        let disk = metrics.disk.unwrap_or_default();
        let process = metrics.process.unwrap_or_default();

        Self {
            agent_id: event.agent_id,
            timestamp: event.timestamp,
            cpu_usage_percent: metrics.cpu_usage_percent,
            memory_used_bytes: metrics.memory_used_bytes.to_string(),
            memory_total_bytes: metrics.memory_total_bytes.to_string(),
            network_rx_bytes_per_sec: network.rx_bytes_per_sec.to_string(),
            network_tx_bytes_per_sec: network.tx_bytes_per_sec.to_string(),
            disk_read_bytes_per_sec: disk.read_bytes_per_sec.to_string(),
            disk_write_bytes_per_sec: disk.write_bytes_per_sec.to_string(),
            process_cpu_percent: process.cpu_usage_percent,
            process_memory_bytes: process.memory_bytes.to_string(),
            process_uptime_seconds: process.uptime_seconds as i32,
        }
    }
}
