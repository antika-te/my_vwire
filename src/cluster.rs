//! HA 集群管理器
//! 
//! 支持多實例高可用部署

use anyhow::Result;
use log::{info, debug, warn, error};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};

/// 集群節點信息
#[derive(Clone, Debug)]
pub struct ClusterNode {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub status: NodeStatus,
    pub last_heartbeat: u64,
}

/// 節點狀態
#[derive(Clone, Debug, PartialEq)]
pub enum NodeStatus {
    Active,
    Standby,
    Failed,
}

/// HA 集群管理器
pub struct ClusterManager {
    /// 本地節點 ID
    node_id: String,
    /// 集群模式
    mode: ClusterMode,
    /// 集群節點列表
    nodes: Arc<RwLock<Vec<ClusterNode>>>,
    /// Redis 連接（用於分布式鎖）
    redis_client: Option<redis::Client>,
}

/// 集群模式
#[derive(Clone, Debug)]
pub enum ClusterMode {
    /// 獨立模式
    Standalone,
    /// 主從模式
    ActiveStandby,
    /// 多活模式
    MultiActive,
}

impl ClusterManager {
    /// 創建集群管理器
    pub fn new(node_id: String, mode: ClusterMode, redis_url: Option<String>) -> Self {
        let redis_client = redis_url.and_then(|url| {
            redis::Client::open(url.as_str()).ok()
        });
        
        ClusterManager {
            node_id,
            mode,
            nodes: Arc::new(RwLock::new(Vec::new())),
            redis_client,
        }
    }
    
    /// 啟動集群管理
    pub async fn start(&self) -> Result<()> {
        info!("啟動 HA 集群管理器，模式：{:?}", self.mode);
        
        let self_clone = self.clone();
        tokio::spawn(async move {
            let mut heartbeat_interval = interval(Duration::from_secs(5));
            loop {
                heartbeat_interval.tick().await;
                if let Err(e) = self_clone.send_heartbeat().await {
                    error!("發送 heartbeat 失敗：{}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// 發送心跳
    async fn send_heartbeat(&self) -> Result<()> {
        // TODO: 實現 heartbeat 邏輯
        debug!("發送 heartbeat: {}", self.node_id);
        Ok(())
    }
    
    /// 選舉主節點（Active-Standby 模式）
    pub async fn elect_leader(&self) -> Result<bool> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_connection()?;
            let key = "vwire:leader";
            let won: bool = redis::setnx(&mut conn, key, &self.node_id)?;
            if won {
                info!("當選為 leader: {}", self.node_id);
                Ok(true)
            } else {
                debug!("非 leader 節點");
                Ok(false)
            }
        } else {
            // 無 Redis，假設自己是 leader
            Ok(true)
        }
    }
    
    /// 註冊節點
    pub async fn register_node(&self, node: ClusterNode) {
        let mut nodes = self.nodes.write().await;
        nodes.push(node);
        info!("註冊集群節點：{}", node.id);
    }
    
    /// 獲取活動節點列表
    pub async fn get_active_nodes(&self) -> Vec<ClusterNode> {
        let nodes = self.nodes.read().await;
        nodes.iter()
            .filter(|n| n.status == NodeStatus::Active)
            .cloned()
            .collect()
    }
    
    /// 故障轉移
    pub async fn failover(&self, failed_node_id: &str) -> Result<()> {
        warn!("檢測到節點故障：{}", failed_node_id);
        
        let mut nodes = self.nodes.write().await;
        for node in nodes.iter_mut() {
            if node.id == failed_node_id {
                node.status = NodeStatus::Failed;
                break;
            }
        }
        
        // 如果是 leader 故障，觸發選舉
        if self.mode == ClusterMode::ActiveStandby {
            self.elect_leader().await?;
        }
        
        Ok(())
    }
    
    /// 獲取集群狀態
    pub async fn get_cluster_status(&self) -> ClusterStatus {
        let nodes = self.nodes.read().await;
        let total = nodes.len();
        let active = nodes.iter().filter(|n| n.status == NodeStatus::Active).count();
        let failed = nodes.iter().filter(|n| n.status == NodeStatus::Failed).count();
        
        ClusterStatus {
            total_nodes: total,
            active_nodes: active,
            failed_nodes: failed,
            mode: format!("{:?}", self.mode),
            is_leader: self.elect_leader().await.unwrap_or(false),
        }
    }
}

/// 集群狀態
#[derive(Clone, Debug)]
pub struct ClusterStatus {
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub failed_nodes: usize,
    pub mode: String,
    pub is_leader: bool,
}

impl Clone for ClusterManager {
    fn clone(&self) -> Self {
        ClusterManager {
            node_id: self.node_id.clone(),
            mode: self.mode.clone(),
            nodes: Arc::clone(&self.nodes),
            redis_client: self.redis_client.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cluster_manager_basic() {
        let manager = ClusterManager::new(
            "node-1".to_string(),
            ClusterMode::ActiveStandby,
            None,
        );
        
        let node = ClusterNode {
            id: "node-1".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            status: NodeStatus::Active,
            last_heartbeat: 0,
        };
        
        manager.register_node(node).await;
        
        let status = manager.get_cluster_status().await;
        assert_eq!(status.total_nodes, 1);
        assert_eq!(status.active_nodes, 1);
    }
}
