#![no_std]
#![no_main]

use aya_ebpf::{bindings::*, macros::xdp, programs::XdpContext};
use aya_ebpf_maps::BpfMap;
use aya_ebpf::{EbpfError, maps::BpfHash};

// 定義 XDP 動作
const XDP_PASS: u32 = 2;
const XDP_REDIRECT: u32 = 3;
const XDP_DROP: u32 = 1;

// 定義 AF_XDP socket 的 map
#[map]
static mut SOCKMAP: BpfMap<sockmap::SockMap, u32> = BpfMap::uninit();

// 統計信息 map
#[map]
static mut STATS: BpfHash<StatsKey, StatsValue> = BpfMap::uninit();

#[derive(Clone, Copy)]
#[repr(C)]
struct StatsKey {
    cpu: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct StatsValue {
    total_packets: u64,
    http_packets: u64,
    redirected_packets: u64,
    dropped_packets: u64,
}

/// XDP 程序入口
/// 主要功能:
/// 1. 解析以太網幀和 IP/TCP header
/// 2. 識別目標端口為 80 (HTTP) 的流量
/// 3. 將 HTTP 流量重定向到 userspace 進行廣告過濾
/// 4. 非 HTTP 流量直接放行 (fastpath)
#[xdp]
pub fn vwire_xdp(ctx: XdpContext) -> u32 {
    unsafe {
        // 更新統計信息
        let stats_key = StatsKey { cpu: bpf_get_smp_processor_id() };
        if let Some(stats) = STATS.get_mut(&stats_key) {
            stats.total_packets += 1;
        }

        // 獲取數據包指針和長度
        let data = ctx.data() as *const u8;
        let data_end = ctx.data_end() as *const u8;
        
        // 確保至少有以太網 header (14 bytes)
        if data_end.offset_from(data) < 14 {
            return XDP_PASS;
        }

        // 解析以太網 header
        let eth = data as *const ethhdr;
        let h_proto = (*eth).h_proto;
        
        // 只處理 IPv4 流量 (ETH_P_IP = 0x0800)
        if h_proto != (ETH_P_IP as u16) {
            return XDP_PASS;
        }

        // 確保至少有 IP header (最小 20 bytes)
        let ip_start = data.add(14);
        if data_end.offset_from(ip_start) < 20 {
            return XDP_PASS;
        }

        // 解析 IP header
        let iph = ip_start as *const iphdr;
        let ihl = (*iph).ihl as usize;
        let protocol = (*iph).protocol;
        
        // 只處理 TCP 流量 (IPPROTO_TCP = 6)
        if protocol != IPPROTO_TCP {
            return XDP_PASS;
        }

        // 確保至少有 TCP header (最小 20 bytes)
        let tcp_start = ip_start.add(ihl * 4);
        if data_end.offset_from(tcp_start) < 20 {
            return XDP_PASS;
        }

        // 解析 TCP header
        let tcph = tcp_start as *const tcphdr;
        let dest_port = u16::from_be((*tcph).dest);
        
        // 更新 HTTP 流量統計
        if dest_port == 80 {
            if let Some(stats) = STATS.get_mut(&stats_key) {
                stats.http_packets += 1;
            }
        }

        // 策略：只攔截目標端口為 80 的 HTTP 流量
        // 其他流量全部放行 (包括 HTTPS 443 端口，留待後續擴展)
        if dest_port != 80 {
            return XDP_PASS;
        }

        // TODO: 將 HTTP 流量重定向到 AF_XDP socket
        // 目前先標記為需要處理，後續實現完整的 redirect 邏輯
        
        // 更新重定向統計
        if let Some(stats) = STATS.get_mut(&stats_key) {
            stats.redirected_packets += 1;
        }

        // 暂时返回 PASS，後續會改為 XDP_REDIRECT
        XDP_PASS
    }
}

// 必要的 panic 處理
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
